pub mod config;
pub mod keys;
pub mod screens;

use config::*;
use keys::*;
use screens::*;
use std::process;
use xcb::x;
use xcb::Connection;

xcb::atoms_struct! {
    #[derive(Copy, Clone, Debug)]
    /// Atoms struct for the window manager.
    pub(crate) struct Atoms {
        pub wm_protocols => b"WM_PROTOCOLS" only_if_exists = false,
        pub wm_del_window => b"WM_DELETE_WINDOW" only_if_exists = false,
        pub wm_state => b"WM_STATE" only_if_exists = false,
        pub wm_take_focus => b"WM_TAKE_FOCUS" only_if_exists = false,
        pub net_active_window => b"_NET_ACTIVE_WINDOW" only_if_exists = false,
        pub net_supported => b"_NET_SUPPORTED" only_if_exists = false,
        pub net_wm_name => b"_NET_WM_NAME" only_if_exists = false,
        pub net_wm_state => b"_NET_WM_STATE" only_if_exists = false,
        pub net_wm_fullscreen => b"_NET_WM_STATE_FULLSCREEN" only_if_exists = false,
        pub net_wm_window_type => b"_NET_WM_WINDOW_TYPE" only_if_exists = false,
        pub net_wm_window_type_dialog => b"_NET_WM_WINDOW_TYPE_DIALOG" only_if_exists = false,
        pub net_client_list => b"_NET_CLIENT_LIST" only_if_exists = false,
    }
}

/// The window manager I suppose.
pub struct Lapin {
    pub x_connection: Connection,
    pub config: Config,
    pub keybinds: KeybindSet,
    screens: Vec<Screen>,
    current_scr: usize,
    atoms: Option<Atoms>,
}

impl Lapin {
    /// The first function that should be called: to connect the window manager
    /// to the X server.
    pub fn connect() -> Self {
        let (x_connection, current_scr) =
            Connection::connect(None).expect("Cannot connect to the X server!");
        let config = Config::new();
        let screens = Vec::new();
        let keybinds = KeybindSet::new();
        Lapin {
            x_connection,
            config,
            screens,
            current_scr: current_scr as usize,
            keybinds,
            atoms: None,
        }
    }

    /// Gets the current screen.
    pub fn current_screen(&mut self) -> &mut Screen {
        &mut self.screens[self.current_scr]
    }

    fn handle_x_event(&mut self, event: x::Event, keybinds: &mut KeybindSet) {
        match event {
            x::Event::MapRequest(ev) => {
                println!("map request received");
                self.x_connection.send_request(&x::MapWindow {
                    window: ev.window(),
                });
                self.x_connection.flush().ok();
            }
            x::Event::KeyPress(ev) => {
                println!("button pressed!");
                if let Some(callback) = keybinds.get_callback(ev.detail(), ev.state()) {
                    callback(self);
                };
            }
            other => {
                println!("Received event: {:?}", other);
            }
        }
    }

    /// The main event loop of the window manager.
    fn event_loop(&mut self, keybinds: &mut KeybindSet) -> ! {
        loop {
            let event = self
                .x_connection
                .wait_for_event()
                .expect("Connection to the X server failed!");
            match event {
                xcb::Event::X(ev) => self.handle_x_event(ev, keybinds),
                _ => {}
            }
        }
    }

    pub fn init(&mut self, keybinds: &mut KeybindSet) {
        for screen in self.x_connection.get_setup().roots() {
            self.screens.push(Screen::new(
                &self,
                screen.root(),
                keybinds,
                screen.width_in_pixels(),
                screen.height_in_pixels(),
            ));
        }

        self.atoms = Some(Atoms::intern_all(&self.x_connection).expect("Cannot init atoms!"));

        self.event_loop(keybinds);
    }

    pub fn killfocused(&mut self) {}

    /// Function to spawn a command.
    pub fn spawn(s: &str) {
        let mut iter = s.split_whitespace();
        if let Some(prog) = iter.next() {
            let mut cmd = process::Command::new(prog);
            for arg in iter {
                cmd.arg(arg);
            }
            cmd.spawn().ok();
        }
    }

    /// Function to terminate the window manager process.
    pub fn quit() {
        process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
