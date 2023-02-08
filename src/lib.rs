pub mod config;
pub mod keys;
pub mod screens;

use config::*;
use keys::*;
use screens::*;
use std::process;
use x11rb::connection::Connection;
use x11rb::protocol::xproto as xp;
use x11rb::protocol::Event as XEvent;

/// The window manager I suppose.
pub struct Lapin {
    pub x_connection: x11rb::rust_connection::RustConnection,
    pub config: Config,
    screens: Vec<Screen>,
    current_scr: usize,
    mouse_keymask: Option<xp::KeyButMask>,
    keybinds: KeybindSet,
}

impl Lapin {
    /// The first function that should be called: to connect the window manager
    /// to the X server.
    pub fn connect() -> Self {
        let (x_connection, current_scr) =
            x11rb::connect(None).expect("Cannot connect to the X server!");
        let config = Config::new();
        let screens = Vec::new();
        let keybinds = KeybindSet::new();
        Lapin {
            x_connection,
            config,
            screens,
            current_scr,
            mouse_keymask: None,
            keybinds,
        }
    }

    /// Gets the current screen.
    pub fn current_screen(&mut self) -> &mut Screen {
        &mut self.screens[self.current_scr]
    }

    fn handle_event(&mut self, event: XEvent) -> Result<(), x11rb::errors::ConnectionError> {
        match event {
            XEvent::MapRequest(ev) => {
                println!("map request received");
                xp::map_window(&self.x_connection, ev.window)?;
                self.x_connection.flush()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// The main event loop of the window manager.
    fn event_loop(&mut self) -> ! {
        loop {
            let event = self
                .x_connection
                .wait_for_event()
                .expect("Connection to the X server failed!");
            self.handle_event(event)
                .expect("Connection to the X server failed!");
        }
    }

    pub fn init(&mut self) {
        let (modmask, modbutton) = keys::match_mod(self.config.mouse_modkey);
        self.mouse_keymask = Some(modbutton);

        for screen in &self.x_connection.setup().roots {
            self.screens.push(Screen::new(&self, screen.root, modmask));
        }

        self.event_loop();
    }

    pub fn killfocused(&self) {}
}

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

#[cfg(test)]
mod tests {
    use super::*;
}
