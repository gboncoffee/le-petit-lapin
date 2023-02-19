use crate::layouts::*;

/// General configuration of the window manager.
pub struct Config {
    pub workspaces: &'static [&'static str],
    pub mouse_mod: &'static [&'static str],
    pub border_color: u32,
    pub border_color_focus: u32,
    pub layouts: Vec<Box<dyn Layout>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            workspaces: &["1", "2", "3", "4", "5", "6", "7", "8", "9"],
            mouse_mod: &["Super"],
            border_color: 0xff000000,
            border_color_focus: 0xffffffff,
            layouts: vec![Box::new(Floating::new())],
        }
    }
}
