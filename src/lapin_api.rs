use crate::keys::KeybindSet;
use crate::screens::Screen;
use crate::Atoms;
use crate::Lapin;
use std::process;
use xcb::x;

impl Lapin {
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

    pub fn get_focused_window(&self) -> Option<x::Window> {
        let s = self.current_scr;
        let k = self.screens[s].current_wk;
        if let Some(w) = self.screens[s].workspaces[k].focused {
            Some(self.screens[s].workspaces[k].windows[w])
        } else {
            None
        }
    }

    pub fn killfocused(&mut self) {
        let Some(window) = self.get_focused_window() else { return };
        self.x_connection.send_request(&x::DestroyWindow { window });
        self.x_connection.flush().ok();
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
}
