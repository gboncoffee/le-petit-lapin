use crate::config::Config;
use crate::keys::KeybindSet;
use crate::screens::Screen;
use crate::Lapin;
use std::process;
use xcb::x;
use xcb::Connection;

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
        }
    }

    pub fn init(&mut self, keybinds: &mut KeybindSet) {
        for screen in self.x_connection.get_setup().roots() {
            self.screens
                .push(Screen::new(&self, screen.root(), keybinds));
        }

        self.x_connection.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: self.current_screen().root,
            time: x::CURRENT_TIME,
        });

        self.main_event_loop(keybinds);
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

    pub fn nextwin(&mut self) {
        self.change_win(false);
    }

    pub fn prevwin(&mut self) {
        self.change_win(true);
    }

    pub fn next_layout(&mut self) {
        self.change_layout(false);
    }

    pub fn prev_layout(&mut self) {
        self.change_layout(true);
    }

    pub fn goto_workspace(&mut self, wk: usize) {
        if (wk - 1) < self.current_screen().workspaces.len() {
            self.change_workspace(wk - 1);
        }
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
