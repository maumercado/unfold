//! Modal dialogs: keyboard shortcuts, about, update check and simple messages.
//!
//! gpui-component draws dialogs in a layer above the window content
//! (`window.open_dialog`), with Escape and click-outside to close built in.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::Dialog;
use gpui_kit::component::{ActiveTheme, WindowExt, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::settings;
use crate::theme::Colors;
use crate::update_check::UpdateCheck;

fn colors(cx: &App) -> Colors {
    Colors::for_theme(settings::theme(cx))
}

fn ok_button(id: &'static str) -> Button {
    Button::new(id)
        .label("OK")
        .primary()
        .on_click(|_, window, cx| window.close_dialog(cx))
}

/// Open the shortcuts dialog, or close it if it is already open (Cmd+/).
pub fn toggle_shortcuts(window: &mut Window, cx: &mut App) {
    if window.has_active_dialog(cx) {
        window.close_dialog(cx);
        return;
    }
    let mac = cfg!(target_os = "macos");
    let (cmd, shift, opt) = if mac {
        ("⌘", "⇧", "⌥")
    } else {
        ("Ctrl+", "Shift+", "Alt+")
    };
    let sections: Vec<(&str, Vec<(String, &str)>)> = vec![
        (
            "File",
            vec![
                (format!("{cmd}O"), "Open file"),
                (format!("{cmd}N"), "New window"),
                (format!("{shift}{cmd}E"), "Open in external editor"),
            ],
        ),
        (
            "Edit",
            vec![
                (format!("{cmd}C"), "Copy selected value"),
                (format!("{shift}{cmd}C"), "Copy key name"),
                (format!("{opt}{cmd}C"), "Copy node path"),
            ],
        ),
        (
            "Search",
            vec![
                (format!("{cmd}F"), "Focus search"),
                ("Enter".into(), "Next result"),
                (format!("{shift}Enter"), "Previous result"),
                ("Escape".into(), "Clear search"),
            ],
        ),
        (
            "Navigate",
            vec![
                ("↑ ↓".into(), "Select previous / next row"),
                ("→ ←".into(), "Expand / collapse"),
            ],
        ),
        (
            "View",
            vec![
                (format!("{cmd}T"), "Toggle theme"),
                (format!("{cmd}/"), "Toggle this help"),
            ],
        ),
    ];

    window.open_dialog(cx, move |dialog, _, cx| {
        let c = colors(cx);
        let mono = cx.theme().mono_font_family.clone();
        let mut body = v_flex().gap_1().text_sm();
        for (title, rows) in &sections {
            body = body.child(div().mt_2().text_color(c.key).child(title.to_string()));
            for (keys, desc) in rows {
                body = body.child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .w(px(120.))
                                .font_family(mono.clone())
                                .child(keys.clone()),
                        )
                        .child(div().text_color(c.text_secondary).child(desc.to_string())),
                );
            }
        }
        dialog
            .title("Keyboard Shortcuts")
            .w(px(380.))
            .child(body)
            .child(
                div()
                    .mt_3()
                    .text_xs()
                    .text_color(c.text_secondary)
                    .child(format!("Press Escape or {cmd}/ to close")),
            )
    });
}

pub fn about(window: &mut Window, cx: &mut App) {
    window.open_dialog(cx, |dialog: Dialog, _, cx| {
        let c = colors(cx);
        dialog
            .title("Unfold")
            .w(px(320.))
            .child(
                v_flex()
                    .gap_1()
                    .text_sm()
                    .child(format!("Version {}", env!("CARGO_PKG_VERSION")))
                    .child(
                        div()
                            .text_color(c.text_secondary)
                            .child("A high-performance JSON viewer"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(c.text_secondary)
                            .child("Copyright 2025 Mauricio Mercado"),
                    ),
            )
            .footer(h_flex().justify_end().child(ok_button("about-ok")))
    });
}

/// A title, a message and an OK button. `ok` picks the title color.
pub fn message(window: &mut Window, cx: &mut App, title: &'static str, msg: String, ok: bool) {
    window.open_dialog(cx, move |dialog, _, cx| {
        let c = colors(cx);
        dialog
            .title(
                div()
                    .text_color(if ok { c.success } else { c.error })
                    .child(title),
            )
            .w(px(380.))
            .child(
                div()
                    .text_sm()
                    .text_color(c.text_secondary)
                    .child(msg.clone()),
            )
            .footer(h_flex().justify_end().child(ok_button("message-ok")))
    });
}

pub fn update_check(state: &UpdateCheck, window: &mut Window, cx: &mut App) {
    let state = state.clone();
    window.open_dialog(cx, move |dialog, _, cx| {
        let c = colors(cx);
        let (title, body, footer) = match &state {
            UpdateCheck::Checking => (
                "Checking for Updates",
                div().child("Contacting GitHub..."),
                None,
            ),
            UpdateCheck::UpToDate => (
                "You're Up to Date",
                div().child(format!(
                    "Unfold {} is the latest version.",
                    env!("CARGO_PKG_VERSION")
                )),
                Some(h_flex().justify_end().child(ok_button("update-ok"))),
            ),
            UpdateCheck::UpdateAvailable {
                version,
                release_url,
            } => {
                let url = release_url.clone();
                (
                    "Update Available",
                    div().child(format!("A new version ({}) is available!", version)),
                    Some(
                        h_flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                Button::new("update-later")
                                    .label("Later")
                                    .on_click(|_, window, cx| window.close_dialog(cx)),
                            )
                            .child(
                                Button::new("update-download")
                                    .label("Download")
                                    .primary()
                                    .on_click(move |_, window, cx| {
                                        cx.open_url(&url);
                                        window.close_dialog(cx);
                                    }),
                            ),
                    ),
                )
            }
            UpdateCheck::Error(msg) => (
                "Update Check Failed",
                div().text_color(c.error).child(msg.clone()),
                Some(h_flex().justify_end().child(ok_button("update-ok"))),
            ),
        };
        dialog
            .title(title)
            .w(px(380.))
            .child(div().text_sm().text_color(c.text_secondary).child(body))
            .when_some(footer, |d, f| d.footer(f))
    });
}
