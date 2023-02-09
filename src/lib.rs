pub mod config;
pub mod keys;
pub mod screens;

use config::*;
use keys::*;
use screens::*;
use std::process;
use xcb::x;
use xcb::Connection;

/// The window manager I suppose.
pub struct Lapin {
    pub x_connection: Connection,
    pub config: Config,
    pub keybinds: KeybindSet,
    screens: Vec<Screen>,
    current_scr: usize,
    mouse_keymask: Option<x::KeyButMask>,
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
            mouse_keymask: None,
            keybinds,
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
        let (modmask, modbutton) = keys::match_mod(self.config.mouse_modkey);
        self.mouse_keymask = Some(modbutton);

        for screen in self.x_connection.get_setup().roots() {
            self.screens.push(Screen::new(
                &self,
                screen.root(),
                modmask,
                keybinds,
                screen.width_in_pixels(),
                screen.height_in_pixels(),
            ));
        }

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
