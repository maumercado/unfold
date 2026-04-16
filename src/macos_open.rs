#![cfg(target_os = "macos")]
#![allow(unexpected_cfgs)]

use objc::declare::MethodImplementation;
use objc::runtime::{self, BOOL, Class, NO, Object, Sel, YES};
use objc::{Encode, EncodeArguments, class, msg_send, sel, sel_impl};
use std::ffi::{CStr, CString, c_char};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

const NS_APPLICATION_DELEGATE_REPLY_SUCCESS: usize = 0;
const NS_APPLICATION_DELEGATE_REPLY_FAILURE: usize = 2;

static APP_READY: AtomicBool = AtomicBool::new(false);
static OPEN_FILE_HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);
static PENDING_FILES: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();

fn pending_files() -> &'static Mutex<Vec<PathBuf>> {
    PENDING_FILES.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn install_open_file_handler() {
    if OPEN_FILE_HANDLER_INSTALLED.load(Ordering::SeqCst) {
        return;
    }

    unsafe {
        let Some(delegate_class) = winit_application_delegate_class() else {
            return;
        };

        install_delegate_methods(delegate_class as *const _ as *mut _);
        OPEN_FILE_HANDLER_INSTALLED.store(true, Ordering::SeqCst);
    }
}

pub(crate) fn take_pending_open_files() -> Vec<PathBuf> {
    let mut pending = pending_files()
        .lock()
        .expect("pending macOS open-file queue poisoned");

    std::mem::take(&mut *pending)
}

pub(crate) fn mark_app_ready() {
    APP_READY.store(true, Ordering::SeqCst);
}

unsafe fn winit_application_delegate_class() -> Option<&'static Class> {
    unsafe { current_application_delegate_class() }
        .or_else(|| Class::get("WinitApplicationDelegate"))
}

unsafe fn current_application_delegate_class() -> Option<&'static Class> {
    let app: *mut Object = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    if app.is_null() {
        return None;
    }

    let delegate: *mut Object = unsafe { msg_send![app, delegate] };
    if delegate.is_null() {
        return None;
    }

    let delegate_class = unsafe { runtime::object_getClass(delegate) };
    if delegate_class.is_null() {
        None
    } else {
        Some(unsafe { &*delegate_class })
    }
}

unsafe fn install_delegate_methods(delegate_class: *mut Class) {
    unsafe {
        add_method_if_missing(
            delegate_class,
            sel!(application:openURLs:),
            application_open_urls as extern "C" fn(&Object, Sel, *mut Object, *mut Object),
        );
        add_method_if_missing(
            delegate_class,
            sel!(application:openFile:),
            application_open_file as extern "C" fn(&Object, Sel, *mut Object, *mut Object) -> BOOL,
        );
        add_method_if_missing(
            delegate_class,
            sel!(application:openFiles:),
            application_open_files as extern "C" fn(&Object, Sel, *mut Object, *mut Object),
        );
    }
}

unsafe fn add_method_if_missing<F>(delegate_class: *mut Class, selector: Sel, implementation: F)
where
    F: MethodImplementation<Callee = Object>,
{
    if unsafe { runtime::class_getInstanceMethod(delegate_class, selector) }.is_null() {
        let types = method_type_encoding::<F::Ret, F::Args>();
        let _ = unsafe {
            runtime::class_addMethod(
                delegate_class,
                selector,
                implementation.imp(),
                types.as_ptr(),
            )
        };
    }
}

fn method_type_encoding<R, A>() -> CString
where
    R: Encode,
    A: EncodeArguments,
{
    let mut types = R::encode().as_str().to_owned();
    types.push_str(<*mut Object>::encode().as_str());
    types.push_str(Sel::encode().as_str());

    for encoding in A::encodings().as_ref() {
        types.push_str(encoding.as_str());
    }

    CString::new(types).expect("Objective-C method encoding contained null byte")
}

extern "C" fn application_open_urls(
    _this: &Object,
    _cmd: Sel,
    _app: *mut Object,
    urls: *mut Object,
) {
    let paths = unsafe { extract_paths_from_url_array(urls) };
    route_opened_paths(paths);
}

extern "C" fn application_open_file(
    _this: &Object,
    _cmd: Sel,
    _app: *mut Object,
    filename: *mut Object,
) -> BOOL {
    let Some(path) = (unsafe { nsstring_to_string(filename) }).map(PathBuf::from) else {
        return NO;
    };

    route_opened_paths(vec![path]);
    YES
}

extern "C" fn application_open_files(
    _this: &Object,
    _cmd: Sel,
    app: *mut Object,
    filenames: *mut Object,
) {
    let paths = unsafe { extract_paths_from_string_array(filenames) };
    let reply = if paths.is_empty() {
        NS_APPLICATION_DELEGATE_REPLY_FAILURE
    } else {
        route_opened_paths(paths);
        NS_APPLICATION_DELEGATE_REPLY_SUCCESS
    };

    if !app.is_null() {
        unsafe {
            let _: () = msg_send![app, replyToOpenOrPrint: reply];
        }
    }
}

fn route_opened_paths(paths: Vec<PathBuf>) {
    if paths.is_empty() {
        return;
    }

    if APP_READY.load(Ordering::SeqCst) {
        for path in paths {
            crate::spawn_unfold_process(Some(path));
        }
    } else {
        let mut pending = pending_files()
            .lock()
            .expect("pending macOS open-file queue poisoned");

        pending.extend(paths);
    }
}

unsafe fn extract_paths_from_url_array(urls: *mut Object) -> Vec<PathBuf> {
    if urls.is_null() {
        return Vec::new();
    }

    let count: usize = unsafe { msg_send![urls, count] };
    let mut paths = Vec::with_capacity(count);

    for index in 0..count {
        let url: *mut Object = unsafe { msg_send![urls, objectAtIndex: index] };
        if let Some(path) = unsafe { nsurl_to_path(url) } {
            paths.push(path);
        }
    }

    paths
}

unsafe fn extract_paths_from_string_array(strings: *mut Object) -> Vec<PathBuf> {
    if strings.is_null() {
        return Vec::new();
    }

    let count: usize = unsafe { msg_send![strings, count] };
    let mut paths = Vec::with_capacity(count);

    for index in 0..count {
        let string: *mut Object = unsafe { msg_send![strings, objectAtIndex: index] };
        if let Some(path) = unsafe { nsstring_to_string(string) }.map(PathBuf::from) {
            paths.push(path);
        }
    }

    paths
}

unsafe fn nsurl_to_path(url: *mut Object) -> Option<PathBuf> {
    if url.is_null() {
        return None;
    }

    let is_file_url: BOOL = unsafe { msg_send![url, isFileURL] };
    if is_file_url == NO {
        return None;
    }

    let path: *mut Object = unsafe { msg_send![url, path] };
    unsafe { nsstring_to_string(path) }.map(PathBuf::from)
}

unsafe fn nsstring_to_string(ns_string: *mut Object) -> Option<String> {
    if ns_string.is_null() {
        return None;
    }

    let utf8_ptr: *const c_char = unsafe { msg_send![ns_string, UTF8String] };
    if utf8_ptr.is_null() {
        return None;
    }

    Some(
        unsafe { CStr::from_ptr(utf8_ptr) }
            .to_string_lossy()
            .into_owned(),
    )
}
