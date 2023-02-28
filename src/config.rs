//! General configuration of the window manager.

use crate::layouts::*;
use crate::rules::*;

/// General configuration of the window manager.
pub struct Config {
    /// List of the workspaces names. Will be used to create then automatically
    /// on `Lapin::init()`.
    pub workspaces: &'static [&'static str],
    /// Modifier to use on mouse callbacks.
    pub mouse_mod: &'static [&'static str],
    /// Border color of windows in the form ARGB.
    pub border_color: u32,
    /// Border color of focused windows in the form ARGB.
    pub border_color_focus: u32,
    /// Border width of ool windows.
    pub border_width: u32,
    /// Layouts to use.
    pub layouts: Vec<Box<dyn Layout>>,
    /// Rules to apply to windows on spawn.
    pub rules: Vec<Rule>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            workspaces: &["1", "2", "3", "4", "5", "6", "7", "8", "9"],
            mouse_mod: &["Super"],
            border_color: 0xff000000,
            border_color_focus: 0xffffffff,
            border_width: 4,
            layouts: vec![
                Box::new(Tiling::new()),
                Box::new(Maximized::new()),
                Box::new(Floating::new()),
            ],
            rules: vec![],
        }
    }
}
