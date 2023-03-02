//! General configuration of the window manager.

use crate::layouts::*;
use crate::rules::*;

/// General configuration of the window manager.
pub struct Config {
    /// List of the workspaces names. Will be used to create then
    /// automatically on `Lapin::init()`. Defaults to `[ "1", "2",
    /// "3", "4", "5", "6", "7", "8", "9" ]`.
    pub workspaces: &'static [&'static str],
    /// Modifier to use on mouse callbacks. Defaults to `&["Super"]`.
    pub mouse_mod: &'static [&'static str],
    /// Border color of windows in the form ARGB. Defaults to
    /// `0xff000000`.
    pub border_color: u32,
    /// Border color of focused windows in the form ARGB. Defaults to
    /// `0xffffffff`.
    pub border_color_focus: u32,
    /// Border width of ool windows. Defaults to `4`.
    pub border_width: u32,
    /// Layouts to use. Defaults to the three built-in layouts with
    /// default configs.
    pub layouts: Vec<Box<dyn Layout>>,
    /// Rules to apply to windows on spawn. No rule by default.
    pub rules: Vec<Rule>,
    /// If hovering a window should raise it (make it above other
    /// windows). If `false`, it'll just make it focused. Changing the
    /// focus with the keyboard always raise the window. Defaults to
    /// `true`.
    pub mouse_raises_window: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            workspaces: &["1", "2", "3", "4", "5", "6", "7", "8", "9"],
            mouse_mod: &["Super"],
            border_color: 0xff000000,
            border_color_focus: 0xffffffff,
            border_width: 4,
            mouse_raises_window: true,
            layouts: vec![
                Box::new(Tiling::new()),
                Box::new(Maximized::new()),
                Box::new(Floating::new()),
            ],
            rules: vec![],
        }
    }
}
