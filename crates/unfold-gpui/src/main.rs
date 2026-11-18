//! Unfold on GPUI (via gpui-kit). Work in progress: see docs/gpui-migration-plan.md.
//!
//! One process, many windows. Each window holds its own `Viewer`. Opening a
//! file from Finder, the Dock or a drop reuses an empty window if there is
//! one, otherwise it opens a new window, cascaded from the active one.

// No console window on Windows when launched from Explorer.
#![windows_subsystem = "windows"]

mod actions;
mod dialogs;
mod document;
mod menus;
mod platform;
mod row;
mod settings;
mod theme;
mod update_check;
mod viewer;

use std::path::PathBuf;

use futures::StreamExt as _;
use futures::channel::mpsc;
use gpui_kit::*;

use actions::*;
use viewer::Viewer;

const DEFAULT_SIZE: (f32, f32) = (900.0, 700.0);
/// New windows step down and right from the active one by this much.
const CASCADE_OFFSET: f32 = 32.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for arg in &args[1..] {
        match arg.as_str() {
            "-h" | "--help" => return print_help(),
            "-V" | "--version" => return println!("unfold {}", env!("UNFOLD_VERSION")),
            _ => {}
        }
    }
    let cli_files: Vec<PathBuf> = args[1..]
        .iter()
        .filter(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .collect();

    let app = gpui_kit::application().with_assets(gpui_kit::assets::Assets);

    // macOS delivers Finder "Open With", Dock drops and double clicks as URLs,
    // possibly before `run` starts. Queue them and handle them once the app
    // is ready. This replaces the Iced app's `macos_open.rs` delegate hook.
    let (open_tx, mut open_rx) = mpsc::unbounded::<Vec<String>>();
    app.on_open_urls(move |urls| {
        let _ = open_tx.unbounded_send(urls);
    });

    app.run(move |cx: &mut App| {
        gpui_kit::init(cx);
        settings::init(cx);
        actions::bind_keys(cx);
        menus::install(cx);
        register_app_actions(cx);

        // Quit when the last window closes; each window used to be its own
        // process, so this keeps the old behavior.
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        if cli_files.is_empty() {
            open_window(None, cx);
        }
        for path in cli_files.iter().cloned() {
            open_window(Some(path), cx);
        }

        cx.spawn(async move |cx| {
            while let Some(urls) = open_rx.next().await {
                cx.update(|cx| {
                    for path in urls.iter().filter_map(|u| file_url_to_path(u)) {
                        open_path_somewhere(path, cx);
                    }
                });
            }
        })
        .detach();

        cx.activate(true);
    });
}

fn print_help() {
    println!("Unfold - A high-performance JSON viewer\n");
    println!("USAGE:\n    unfold [FILE]...\n");
    println!("ARGS:\n    <FILE>    JSON file to open (optional)\n");
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
}

/// Commands that work without any window, e.g. from the menu bar after the
/// last window closed on macOS.
fn register_app_actions(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &NewWindow, cx| {
        open_window(None, cx);
    });
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|_: &ToggleTheme, cx| settings::toggle_theme(None, cx));
    // With no window open, "Open..." opens one and asks for a file there.
    cx.on_action(|_: &OpenFile, cx| {
        if let Some((window, _)) = open_window(None, cx) {
            window
                .update(cx, |_, window, cx| {
                    window.dispatch_action(Box::new(OpenFile), cx)
                })
                .ok();
        }
    });
    cx.on_action(|_: &Minimize, cx| {
        if let Some(window) = cx.active_window() {
            window
                .update(cx, |_, window, _| window.minimize_window())
                .ok();
        }
    });
    cx.on_action(|_: &Zoom, cx| {
        if let Some(window) = cx.active_window() {
            window.update(cx, |_, window, _| window.zoom_window()).ok();
        }
    });
}

/// Turn `file:///path/to/a%20b.json` into a path. Non-file URLs are ignored.
fn file_url_to_path(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?, 16)
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    Some(PathBuf::from(String::from_utf8(out).ok()?))
}

/// Open `path` in an empty window if one exists, otherwise in a new window.
fn open_path_somewhere(path: PathBuf, cx: &mut App) {
    for handle in cx.windows() {
        let Some(handle) = handle.downcast::<gpui_kit::base::Root>() else {
            continue;
        };
        let viewer = handle
            .read(cx)
            .ok()
            .and_then(|root| root.view().clone().downcast::<Viewer>().ok());
        if let Some(viewer) = viewer
            && viewer.read(cx).is_empty()
        {
            let path = path.clone();
            handle
                .update(cx, |_, window, cx| {
                    viewer.update(cx, |v, cx| v.open_path(path, window, cx));
                    window.activate_window();
                })
                .ok();
            return;
        }
    }
    open_window(Some(path), cx);
}

/// Open a new window, optionally loading `path` into it.
pub fn open_window(
    path: Option<PathBuf>,
    cx: &mut App,
) -> Option<(AnyWindowHandle, Entity<Viewer>)> {
    let size = size(px(DEFAULT_SIZE.0), px(DEFAULT_SIZE.1));
    let bounds = match cx
        .active_window()
        .and_then(|w| w.update(cx, |_, w, _| w.bounds()).ok())
    {
        Some(active) => Bounds::new(
            active.origin + point(px(CASCADE_OFFSET), px(CASCADE_OFFSET)),
            active.size,
        ),
        None => Bounds::centered(None, size, cx),
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Unfold - JSON Viewer".into()),
            ..Default::default()
        }),
        app_id: Some("com.codigo.unfold".into()),
        window_min_size: Some(gpui_kit::size(px(400.), px(300.))),
        ..Default::default()
    };
    let result = gpui_kit::open_window(options, cx, |window, cx| {
        let viewer = cx.new(|cx| Viewer::new(window, cx));
        viewer.update(cx, |v, cx| {
            window.focus(v.focus_handle(), cx);
            if let Some(path) = path {
                v.open_path(path, window, cx);
            }
        });
        viewer
    });
    result.ok()
}

#[cfg(test)]
mod tests {
    use super::file_url_to_path;
    use std::path::PathBuf;

    #[test]
    fn file_urls_are_decoded() {
        assert_eq!(
            file_url_to_path("file:///tmp/a%20b.json"),
            Some(PathBuf::from("/tmp/a b.json"))
        );
        assert_eq!(file_url_to_path("https://example.com"), None);
        assert_eq!(
            file_url_to_path("file:///tmp/caf%C3%A9.json"),
            Some(PathBuf::from("/tmp/café.json"))
        );
    }
}
