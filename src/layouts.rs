use xcb::x;
use xcb::Connection;

pub trait Layout {
    fn newwin(&self, windows: &mut dyn Iterator<Item = &x::Window>, con: &Connection);
    fn delwin(
        &self,
        windows: &mut dyn Iterator<Item = &x::Window>,
        current: Option<usize>,
        con: &Connection,
    );
    fn reload(&self, windows: &mut dyn Iterator<Item = &x::Window>, con: &Connection);
    fn changewin(
        &self,
        windows: &mut dyn Iterator<Item = &x::Window>,
        number: usize,
        con: &Connection,
        previous: bool,
    );
    fn allow_motions(&self) -> bool;
    fn draw_borders(&self) -> bool;

    fn name(&self) -> &'static str;
}

pub struct Floating {
    pub borders: bool,
    pub name: &'static str,
}

impl Floating {
    pub fn new() -> Self {
        Floating {
            borders: true,
            name: "Floating",
        }
    }
}

impl Layout for Floating {
    fn name(&self) -> &'static str {
        self.name
    }

    fn allow_motions(&self) -> bool {
        true
    }

    fn draw_borders(&self) -> bool {
        self.borders
    }

    fn newwin(&self, _windows: &mut dyn Iterator<Item = &x::Window>, _con: &Connection) {}
    fn delwin(
        &self,
        _windows: &mut dyn Iterator<Item = &x::Window>,
        _current: Option<usize>,
        _con: &Connection,
    ) {
    }
    fn reload(&self, _windows: &mut dyn Iterator<Item = &x::Window>, _con: &Connection) {}
    fn changewin(
        &self,
        _windows: &mut dyn Iterator<Item = &x::Window>,
        _number: usize,
        _con: &Connection,
        _previous: bool,
    ) {
    }
}

pub struct Tiling {}

pub struct Maximized {
    pub name: &'static str,
    pub width: u16,
    pub height: u16,
    pub pos: (i32, i32),
    pub borders: bool,
}

impl Maximized {
    pub fn new(width: u16, height: u16) -> Maximized {
        Maximized {
            width,
            height,
            name: "Maximized",
            pos: (0, 0),
            borders: false,
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

    fn draw_borders(&self) -> bool {
        self.borders
    }

    fn newwin(&self, windows: &mut dyn Iterator<Item = &x::Window>, con: &Connection) {
        let window = *windows.next().unwrap();
        let list = [
            x::ConfigWindow::X(self.pos.0),
            x::ConfigWindow::Y(self.pos.1),
            x::ConfigWindow::Width(self.width as u32),
            x::ConfigWindow::Height(self.height as u32),
        ];
        con.send_request(&x::ConfigureWindow {
            window,
            value_list: &list,
        });
    }
    fn reload(&self, windows: &mut dyn Iterator<Item = &x::Window>, con: &Connection) {
        let list = [
            x::ConfigWindow::X(self.pos.0),
            x::ConfigWindow::Y(self.pos.1),
            x::ConfigWindow::Width(self.width as u32),
            x::ConfigWindow::Height(self.height as u32),
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
        _windows: &mut dyn Iterator<Item = &x::Window>,
        _current: Option<usize>,
        _con: &Connection,
    ) {
    }
    fn changewin(
        &self,
        _windows: &mut dyn Iterator<Item = &x::Window>,
        _number: usize,
        _con: &Connection,
        _previous: bool,
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
