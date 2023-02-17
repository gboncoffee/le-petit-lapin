use std::slice::Iter;
use xcb::x;
use xcb::Connection;

pub trait Layout {
    fn newwin(&self, windows: &mut Iter<x::Window>, con: &Connection, width: u32, height: u32);
    fn delwin(
        &self,
        windows: &mut Iter<x::Window>,
        current: Option<usize>,
        con: &Connection,
        width: u32,
        height: u32,
    );
    fn reload(&self, windows: &mut Iter<x::Window>, con: &Connection, width: u32, height: u32);
    fn changewin(
        &self,
        windows: &mut Iter<x::Window>,
        number: usize,
        con: &Connection,
        previous: bool,
        width: u32,
        height: u32,
    );
    fn allow_motions(&self) -> bool;
    fn border_width(&self) -> u32;

    fn name(&self) -> &'static str;
}

pub struct Floating {
    pub borders: u32,
    pub name: &'static str,
}

impl Floating {
    pub fn new() -> Self {
        Floating {
            borders: 4,
            name: "Floating",
        }
    }
}

impl Layout for Floating {
    fn newwin(&self, _windows: &mut Iter<x::Window>, _con: &Connection, _width: u32, _height: u32) {
    }
    fn delwin(
        &self,
        _windows: &mut Iter<x::Window>,
        _current: Option<usize>,
        _con: &Connection,
        _width: u32,
        _height: u32,
    ) {
    }
    fn reload(&self, _windows: &mut Iter<x::Window>, _con: &Connection, _width: u32, _height: u32) {
    }
    fn changewin(
        &self,
        _windows: &mut Iter<x::Window>,
        _number: usize,
        _con: &Connection,
        _previous: bool,
        _width: u32,
        _height: u32,
    ) {
    }
    fn allow_motions(&self) -> bool {
        true
    }
    fn border_width(&self) -> u32 {
        self.borders
    }
    fn name(&self) -> &'static str {
        self.name
    }
}

pub struct Tiling {
    pub name: &'static str,
    pub borders: u32,
    pub master_factor: f32,
}

impl Tiling {
    pub fn new() -> Tiling {
        Tiling {
            name: "Tiling",
            borders: 4,
            master_factor: 1.0 / 2.0,
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
    fn border_width(&self) -> u32 {
        self.borders
    }

    fn reload(&self, windows: &mut Iter<x::Window>, con: &Connection, width: u32, height: u32) {
        let n_wins = windows.len();
        if n_wins == 0 {
            return;
        } else if n_wins == 1 {
            let list = [
                x::ConfigWindow::X(0),
                x::ConfigWindow::Y(0),
                x::ConfigWindow::Width(width - (self.borders * 2)),
                x::ConfigWindow::Height(height - (self.borders * 2)),
            ];
            con.send_request(&x::ConfigureWindow {
                window: *windows.next().unwrap(),
                value_list: &list,
            });
        } else {
            let list = [
                x::ConfigWindow::X(0),
                x::ConfigWindow::Y(0),
                x::ConfigWindow::Width(
                    (((width as f32) * self.master_factor) as u32) - (self.borders * 2),
                ),
                x::ConfigWindow::Height(height - (self.borders * 2)),
            ];
            con.send_request(&x::ConfigureWindow {
                window: *windows.next().unwrap(),
                value_list: &list,
            });
            let n_slave_wins = n_wins - 1;
            let x = ((width as f32) * self.master_factor) as u32;
            let width = width - (((width as f32) * self.master_factor) as u32) - (self.borders * 2);
            let height =
                (height - (self.borders * 2 * (n_slave_wins as u32))) / (n_slave_wins as u32);
            for (n, window) in windows.enumerate() {
                let y = height * (n as u32) + (self.borders * 2 * (n as u32));
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

    fn newwin(&self, windows: &mut Iter<x::Window>, con: &Connection, width: u32, height: u32) {
        self.reload(windows, con, width, height);
    }
    fn delwin(
        &self,
        windows: &mut Iter<x::Window>,
        _current: Option<usize>,
        con: &Connection,
        width: u32,
        height: u32,
    ) {
        self.reload(windows, con, width, height);
    }
    fn changewin(
        &self,
        _windows: &mut Iter<x::Window>,
        _number: usize,
        _con: &Connection,
        _previous: bool,
        _width: u32,
        _height: u32,
    ) {
    }
}

pub struct Maximized {
    pub name: &'static str,
    pub borders: u32,
    pub gaps: u32,
}

impl Maximized {
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

    fn border_width(&self) -> u32 {
        self.borders
    }

    fn newwin(&self, windows: &mut Iter<x::Window>, con: &Connection, width: u32, height: u32) {
        let window = *windows.next().unwrap();
        let list = [
            x::ConfigWindow::X((self.gaps) as i32),
            x::ConfigWindow::Y((self.gaps) as i32),
            x::ConfigWindow::Width(width - (self.gaps * 2) - (self.borders * 2)),
            x::ConfigWindow::Height(height - (self.gaps * 2) - (self.borders * 2)),
        ];
        con.send_request(&x::ConfigureWindow {
            window,
            value_list: &list,
        });
    }
    fn reload(&self, windows: &mut Iter<x::Window>, con: &Connection, width: u32, height: u32) {
        let list = [
            x::ConfigWindow::X((self.gaps + self.borders) as i32),
            x::ConfigWindow::Y((self.gaps + self.borders) as i32),
            x::ConfigWindow::Width(width - (self.gaps * 2) - (self.borders * 2)),
            x::ConfigWindow::Height(height - (self.gaps * 2) - (self.borders * 2)),
        ];
        for window in windows {
            con.send_request(&x::ConfigureWindow {
                window: *window,
                value_list: &list,
            });
        }
        con.flush().ok();
    }
    fn delwin(
        &self,
        _windows: &mut Iter<x::Window>,
        _current: Option<usize>,
        _con: &Connection,
        _width: u32,
        _height: u32,
    ) {
    }
    fn changewin(
        &self,
        _windows: &mut Iter<x::Window>,
        _number: usize,
        _con: &Connection,
        _previous: bool,
        _width: u32,
        _height: u32,
    ) {
    }
}

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
