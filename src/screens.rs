use crate::*;
use xcb::x;

/// A physical screen, as detected by xinerama.
pub struct Screen {
    pub workspaces: Vec<Workspace>,
    pub current_wk: usize,
    pub width: u16,
    pub height: u16,
    pub x: i16,
    pub y: i16,
}

impl Screen {
    /// Creates a screen. The `Lapin::init()` should call it for every monitor.
    /// Only use it manually if you know what you're doing.
    pub fn new(lapin: &Lapin, width: u16, height: u16, x: i16, y: i16) -> Self {
        let mut workspaces = Vec::with_capacity(lapin.config.workspaces.len());
        for workspace in lapin.config.workspaces {
            workspaces.push(Workspace::new(workspace));
        }

        lapin.x_connection.flush().ok();

        Screen {
            workspaces,
            current_wk: 0,
            width,
            height,
            x,
            y,
        }
    }

    /// Gets the current workspace struct of the screen.
    pub fn current_workspace(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current_wk]
    }
}

/// A virtual workspace.
pub struct Workspace {
    pub name: &'static str,
    pub focused: Option<usize>,
    pub ool_focus: bool,
    pub windows: Vec<x::Window>,
    pub ool_windows: Vec<x::Window>,
    pub layout: usize,
    pub respect_reserved_space: bool,
}

impl Workspace {
    /// Creates a new workspace. The `Lapin::init()` function should create then
    /// based in the config struct. Only create then manually if you know what
    /// you're doing.
    pub fn new(name: &'static str) -> Self {
        Workspace {
            name,
            focused: None,
            ool_focus: false,
            windows: Vec::new(),
            ool_windows: Vec::new(),
            layout: 0,
            respect_reserved_space: true,
        }
    }
}
