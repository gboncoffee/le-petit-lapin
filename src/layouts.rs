//! Default layouts for the window manager and a trait to create new
//! ones.

use std::slice::Iter;
use xcb::x;
use xcb::Connection;

/// A trait that defines a layout of the window manager. Layouts are
/// responsible to send requests to change windows size and position. They're
/// free to do anything, but to better suit with the window manager itself,
/// they should stick to just changing windows size and position.
pub trait Layout {
    /// Called when a window is mapped, except when changing workspaces.
    fn newwin(
        &self,
        windows: &mut Iter<x::Window>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    );
    /// Called when a window is unmaped, except when changing workspaces.
    fn delwin(
        &self,
        windows: &mut Iter<x::Window>,
        current: Option<usize>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    );
    /// Called any time some action requires a full reload of the windows size
    /// and/or position, such as changing workspaces or layouts.
    fn reload(
        &self,
        windows: &mut Iter<x::Window>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    );
    /// Called when the focus was changed.
    fn changewin(
        &self,
        windows: &mut Iter<x::Window>,
        number: usize,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    );
    /// The window manager calls this function when a mouse motion is
    /// performed to check if it should allow it to move and/or resize windows.
    /// Layouts will generally just return `false` unless they're a floating
    /// layout.
    fn allow_motions(&self) -> bool;
    /// The window manager calls this function to get the border size it should
    /// set to windows. Layouts should return 0 to no border.
    fn border_width(&self) -> u16;

    /// Returns the layout name. It's recommended to leave the name as a free
    /// choice of the user.
    fn name(&self) -> &'static str;
}

/// A floating layout. Does nothing with the windows and allows motions.
/// Supports optional borders.
pub struct Floating {
    pub borders: u16,
    pub name: &'static str,
}

impl Floating {
    /// Returns a new floating layout with default configs:
    /// - 4 pixels for borders;
    /// - "Floating" as the name.
    pub fn new() -> Self {
        Floating {
            borders: 4,
            name: "Floating",
        }
    }
}

impl Layout for Floating {
    fn newwin(
        &self,
        _windows: &mut Iter<x::Window>,
        _con: &Connection,
        _width: u16,
        _height: u16,
        _x: i16,
        _y: i16,
    ) {
    }
    fn delwin(
        &self,
        _windows: &mut Iter<x::Window>,
        _current: Option<usize>,
        _con: &Connection,
        _width: u16,
        _height: u16,
        _x: i16,
        _y: i16,
    ) {
    }
    fn reload(
        &self,
        _windows: &mut Iter<x::Window>,
        _con: &Connection,
        _width: u16,
        _height: u16,
        _x: i16,
        _y: i16,
    ) {
    }
    fn changewin(
        &self,
        _windows: &mut Iter<x::Window>,
        _number: usize,
        _con: &Connection,
        _width: u16,
        _height: u16,
        _x: i16,
        _y: i16,
    ) {
    }
    fn allow_motions(&self) -> bool {
        true
    }
    fn border_width(&self) -> u16 {
        self.borders
    }
    fn name(&self) -> &'static str {
        self.name
    }
}

/// A tiling layout, similar to DWM. Supports optional gaps and borders.
pub struct Tiling {
    pub name: &'static str,
    pub borders: u16,
    /// Ratio of the screen used by the master window. Ranges from 0 to 1.
    pub master_factor: f32,
    /// Gaps around and between the windows.
    pub gaps: u16,
}

impl Tiling {
    /// Creates a new tiling layout with default configs:
    /// - 4 pixels for borders;
    /// - 1/2 (0.5) of master factor;
    /// - 4 pixels for gaps;
    /// - "Tiling" as the name.
    pub fn new() -> Tiling {
        Tiling {
            name: "Tiling",
            borders: 4,
            master_factor: 1.0 / 2.0,
            gaps: 4,
        }
    }
}

impl Layout for Tiling {
    fn name(&self) -> &'static str {
        self.name
    }
    fn allow_motions(&self) -> bool {
        false
    }
    fn border_width(&self) -> u16 {
        self.borders
    }

    fn reload(
        &self,
        windows: &mut Iter<x::Window>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    ) {
        let n_wins = windows.len();
        if n_wins == 0 {
            return;
        } else if n_wins == 1 {
            let list = [
                x::ConfigWindow::X((x + (self.gaps as i16)) as i32),
                x::ConfigWindow::Y((y + (self.gaps as i16)) as i32),
                x::ConfigWindow::Width(
                    (width - ((self.gaps * 2) as u16) - ((self.borders * 2) as u16)) as u32,
                ),
                x::ConfigWindow::Height(
                    (height - ((self.gaps * 2) as u16) - ((self.borders * 2) as u16)) as u32,
                ),
            ];
            con.send_request(&x::ConfigureWindow {
                window: *windows.next().unwrap(),
                value_list: &list,
            });
        } else {
            let list = [
                x::ConfigWindow::X((x + (self.gaps as i16)) as i32),
                x::ConfigWindow::Y((y + (self.gaps as i16)) as i32),
                x::ConfigWindow::Width(
                    ((((width as f32) * self.master_factor) as u16)
                        - (((self.gaps as f32) * 1.5) as u16)
                        - (self.borders * 2)) as u32,
                ),
                x::ConfigWindow::Height((height - (self.gaps * 2) - (self.borders * 2)) as u32),
            ];
            con.send_request(&x::ConfigureWindow {
                window: *windows.next().unwrap(),
                value_list: &list,
            });
            let n_slave_wins = n_wins - 1;
            let x = x + (((((width as f32) * self.master_factor) as u16) + (self.gaps / 2)) as i16);
            let width = (width / 2) - (((self.gaps as f32) * 1.5) as u16) - (self.borders * 2);
            let height = (height
                - (self.gaps * (n_slave_wins + 1) as u16)
                - (self.borders * 2 * (n_slave_wins as u16)))
                / (n_slave_wins as u16);
            for (n, window) in windows.enumerate() {
                let y = y
                    + (((height * (n as u16) + (self.borders * 2 * (n as u16)))
                        + (self.gaps * ((n + 1) as u16))) as i16);
                let list = [
                    x::ConfigWindow::X(x as i32),
                    x::ConfigWindow::Y(y as i32),
                    x::ConfigWindow::Width(width as u32),
                    x::ConfigWindow::Height(height as u32),
                ];
                con.send_request(&x::ConfigureWindow {
                    window: *window,
                    value_list: &list,
                });
            }
        }
        con.flush().ok();
    }

    fn newwin(
        &self,
        windows: &mut Iter<x::Window>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    ) {
        self.reload(windows, con, width, height, x, y);
    }
    fn delwin(
        &self,
        windows: &mut Iter<x::Window>,
        _current: Option<usize>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    ) {
        self.reload(windows, con, width, height, x, y);
    }
    fn changewin(
        &self,
        _windows: &mut Iter<x::Window>,
        _number: usize,
        _con: &Connection,
        _width: u16,
        _height: u16,
        _x: i16,
        _y: i16,
    ) {
    }
}

/// A maximized (fullscreen) layout. Windows are drawn above each other.
/// Supports optional gaps and borders. Use 0 to disable then.
pub struct Maximized {
    pub name: &'static str,
    pub borders: u16,
    pub gaps: u16,
}

impl Maximized {
    /// Creates a new maximized layout with default configs:
    /// - No borders nor gaps;
    /// - "Maximized" as the name.
    pub fn new() -> Maximized {
        Maximized {
            name: "Maximized",
            borders: 0,
            gaps: 0,
        }
    }
}

impl Layout for Maximized {
    fn name(&self) -> &'static str {
        self.name
    }

    fn allow_motions(&self) -> bool {
        false
    }

    fn border_width(&self) -> u16 {
        self.borders
    }

    fn newwin(
        &self,
        windows: &mut Iter<x::Window>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    ) {
        let window = *windows.next().unwrap();
        let list = [
            x::ConfigWindow::X((x + (self.gaps as i16)) as i32),
            x::ConfigWindow::Y((y + (self.gaps as i16)) as i32),
            x::ConfigWindow::Width((width - (self.gaps * 2) - (self.borders * 2)) as u32),
            x::ConfigWindow::Height((height - (self.gaps * 2) - (self.borders * 2)) as u32),
        ];
        con.send_request(&x::ConfigureWindow {
            window,
            value_list: &list,
        });
    }
    fn reload(
        &self,
        windows: &mut Iter<x::Window>,
        con: &Connection,
        width: u16,
        height: u16,
        x: i16,
        y: i16,
    ) {
        let list = [
            x::ConfigWindow::X((x + (self.gaps as i16)) as i32),
            x::ConfigWindow::Y((y + (self.gaps as i16)) as i32),
            x::ConfigWindow::Width((width - (self.gaps * 2) - (self.borders * 2)) as u32),
            x::ConfigWindow::Height((height - (self.gaps * 2) - (self.borders * 2)) as u32),
        ];
        for window in windows {
            con.send_request(&x::ConfigureWindow {
                window: *window,
                value_list: &list,
            });
        }
    }
    fn delwin(
        &self,
        _windows: &mut Iter<x::Window>,
        _current: Option<usize>,
        _con: &Connection,
        _width: u16,
        _height: u16,
        _x: i16,
        _y: i16,
    ) {
    }
    fn changewin(
        &self,
        _windows: &mut Iter<x::Window>,
        _number: usize,
        _con: &Connection,
        _width: u16,
        _height: u16,
        _x: i16,
        _y: i16,
    ) {
    }
}

/// Creates a Vec of layouts suitable for use with the window manager.
///
/// # Example
///
/// ```no_run
/// use le_petit_lapin::*;
/// use le_petit_lapin::layouts::*;
/// let mut lapin = Lapin::connect();
/// let tile = Tiling {
///     name: "tile",
///     borders: 4,
///     master_factor: 1.0 / 2.0,
///     gaps: 4,
/// };
/// let max = Maximized {
///     name: "max",
///     borders: 4,
///     gaps: 4,
/// };
/// let float = Floating {
///     name: "float",
///     borders: 4,
/// };
/// lapin.config.layouts = layouts![tile, max, float];
/// ```
#[macro_export]
macro_rules! layouts {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(Box::new($x) as Box<dyn Layout>);
            )*
            temp_vec
        }
    };
}
