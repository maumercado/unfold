//! App-wide settings shared by every window.
//!
//! GPUI *globals* are values stored once per app (`cx.global::<T>()`). The
//! persisted `Config` lives in one, so toggling the theme in one window
//! updates all of them.

use gpui_kit::{App, Global, Window};
use unfold_core::config::Config;

use crate::theme::{self, AppTheme};

struct Settings {
    config: Config,
    /// Off in tests, so they never touch the user's real config file.
    persist: bool,
}

impl Global for Settings {}

pub fn init(cx: &mut App) {
    init_with(Config::load(), true, cx);
}

pub fn init_with(config: Config, persist: bool, cx: &mut App) {
    theme::apply(config.theme, None, cx);
    cx.set_global(Settings { config, persist });
}

pub fn theme(cx: &App) -> AppTheme {
    cx.global::<Settings>().config.theme
}

/// Change the config and save it to disk.
pub fn update(cx: &mut App, f: impl FnOnce(&mut Config)) {
    let settings = cx.global_mut::<Settings>();
    f(&mut settings.config);
    if settings.persist {
        let _ = settings.config.save();
    }
}

pub fn toggle_theme(window: Option<&mut Window>, cx: &mut App) {
    let next = theme::toggled(theme(cx));
    update(cx, |c| c.theme = next);
    theme::apply(next, window, cx);
    cx.refresh_windows();
}
