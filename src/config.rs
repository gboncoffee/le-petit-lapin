/// General configuration of the window manager.
pub struct Config {
    pub workspaces: &'static [&'static str],
    pub mouse_mod: &'static [&'static str],
}

impl Config {
    pub fn new() -> Self {
        Config {
            workspaces: &["1", "2", "3", "4", "5", "6", "7", "8", "9"],
            mouse_mod: &["Super"],
        }
    }
}
