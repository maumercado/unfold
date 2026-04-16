#![cfg(target_os = "macos")]
#![allow(unexpected_cfgs)]

use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::{CStr, c_char};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once, OnceLock};

const K_CORE_EVENT_CLASS: u32 = four_char_code(*b"aevt");
const K_AE_OPEN_DOCUMENTS: u32 = four_char_code(*b"odoc");
const KEY_DIRECT_OBJECT: u32 = four_char_code(*b"----");
const TYPE_FILE_URL: u32 = four_char_code(*b"furl");

static APP_READY: AtomicBool = AtomicBool::new(false);
static INSTALL_HANDLER: Once = Once::new();
static PENDING_FILES: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();

const fn four_char_code(bytes: [u8; 4]) -> u32 {
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | (bytes[3] as u32)
}

fn pending_files() -> &'static Mutex<Vec<PathBuf>> {
    PENDING_FILES.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn install_open_file_handler() {
    INSTALL_HANDLER.call_once(|| unsafe {
        let pool = new_autorelease_pool();

        let handler: *mut Object = msg_send![handler_class(), new];
        let _: *mut Object = msg_send![handler, retain];

        let manager: *mut Object = msg_send![class!(NSAppleEventManager), sharedAppleEventManager];
        let _: () = msg_send![
            manager,
            setEventHandler: handler
            andSelector: sel!(handleAppleEvent:withReplyEvent:)
            forEventClass: K_CORE_EVENT_CLASS
            andEventID: K_AE_OPEN_DOCUMENTS
        ];

        drain_autorelease_pool(pool);
    });
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

extern "C" fn handle_apple_event(
    _this: &Object,
    _cmd: Sel,
    event: *mut Object,
    _reply_event: *mut Object,
) {
    let paths = unsafe { extract_paths_from_event(event) };
    route_opened_paths(paths);
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

fn handler_class() -> &'static Class {
    static CLASS: OnceLock<&'static Class> = OnceLock::new();

    CLASS.get_or_init(|| unsafe {
        if let Some(existing) = Class::get("UnfoldOpenFileHandler") {
            return existing;
        }

        let mut decl = ClassDecl::new("UnfoldOpenFileHandler", class!(NSObject))
            .expect("failed to register UnfoldOpenFileHandler");

        decl.add_method(
            sel!(handleAppleEvent:withReplyEvent:),
            handle_apple_event as extern "C" fn(&Object, Sel, *mut Object, *mut Object),
        );

        decl.register()
    })
}

unsafe fn extract_paths_from_event(event: *mut Object) -> Vec<PathBuf> {
    if event.is_null() {
        return Vec::new();
    }

    let descriptor_list: *mut Object =
        msg_send![event, paramDescriptorForKeyword: KEY_DIRECT_OBJECT];
    if descriptor_list.is_null() {
        return Vec::new();
    }

    let count: i64 = msg_send![descriptor_list, numberOfItems];
    let mut paths = Vec::with_capacity(count.max(0) as usize);

    for index in 1..=count {
        let descriptor: *mut Object = msg_send![descriptor_list, descriptorAtIndex: index];
        if let Some(path) = unsafe { descriptor_to_path(descriptor) } {
            paths.push(path);
        }
    }

    paths
}

unsafe fn descriptor_to_path(descriptor: *mut Object) -> Option<PathBuf> {
    if descriptor.is_null() {
        return None;
    }

    let direct_url: *mut Object = msg_send![descriptor, fileURLValue];
    if let Some(path) = unsafe { nsurl_to_path(direct_url) } {
        return Some(path);
    }

    let file_url_descriptor: *mut Object =
        msg_send![descriptor, coerceToDescriptorType: TYPE_FILE_URL];
    if !file_url_descriptor.is_null() {
        let coerced_url: *mut Object = msg_send![file_url_descriptor, fileURLValue];
        if let Some(path) = unsafe { nsurl_to_path(coerced_url) } {
            return Some(path);
        }
    }

    let string_value: *mut Object = msg_send![descriptor, stringValue];
    unsafe { nsstring_to_string(string_value) }.map(PathBuf::from)
}

unsafe fn nsurl_to_path(url: *mut Object) -> Option<PathBuf> {
    if url.is_null() {
        return None;
    }

    let path: *mut Object = msg_send![url, path];
    unsafe { nsstring_to_string(path) }.map(PathBuf::from)
}

unsafe fn nsstring_to_string(ns_string: *mut Object) -> Option<String> {
    if ns_string.is_null() {
        return None;
    }

    let utf8_ptr: *const c_char = msg_send![ns_string, UTF8String];
    if utf8_ptr.is_null() {
        return None;
    }

    Some(unsafe { CStr::from_ptr(utf8_ptr) }.to_string_lossy().into_owned())
}

unsafe fn new_autorelease_pool() -> *mut Object {
    msg_send![class!(NSAutoreleasePool), new]
}

unsafe fn drain_autorelease_pool(pool: *mut Object) {
    let _: () = msg_send![pool, drain];
}
