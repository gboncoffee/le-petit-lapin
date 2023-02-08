/// General configuration of the window manager.
pub struct Config {
    pub mouse_modkey: &'static str,
    pub workspaces: &'static [&'static str],
}

impl Config {
    pub fn new() -> Self {
        Config {
            mouse_modkey: "Meta",
            workspaces: &["1", "2", "3", "4", "5", "6", "7", "8", "9"],
        }
    }
}
