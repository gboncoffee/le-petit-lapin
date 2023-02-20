//! This module defines a bunch of useful public functions to the `Lapin`
//! struct. Check then on docs for `lapin::Lapin`.
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

    /// The last function that should be called, because it'll start the main
    /// loop and bind keys, efectively never returning.
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

    /// Returns the id of the currently focused window.
    pub fn get_focused_window(&self) -> Option<x::Window> {
        let s = self.current_scr;
        let k = self.screens[s].current_wk;
        if let Some(w) = self.screens[s].workspaces[k].focused {
            Some(self.screens[s].workspaces[k].windows[w])
        } else {
            None
        }
    }

    /// Kills the currently focused window.
    pub fn killfocused(&mut self) {
        let Some(window) = self.get_focused_window() else { return };
        self.x_connection.send_request(&x::DestroyWindow { window });
        self.x_connection.flush().ok();
    }

    /// Changes the focus to the next window of the current workspace.
    pub fn nextwin(&mut self) {
        self.change_win(false);
    }

    /// Changes the focus to the previous window of the current workspace.
    pub fn prevwin(&mut self) {
        self.change_win(true);
    }

    /// Changes to the next layout of the current workspace.
    pub fn next_layout(&mut self) {
        self.change_layout(false);
    }

    /// Changes to the previous layout of the current workspace.
    pub fn prev_layout(&mut self) {
        self.change_layout(true);
    }

    /// Change current workspace.
    pub fn goto_workspace(&mut self, wk: usize) {
        if (wk - 1) < self.current_screen().workspaces.len() {
            self.change_workspace(wk - 1);
        }
    }

    /// Rotate the current workspace stack up.
    pub fn rotate_windows_up(&mut self) {
        if let Some(cur_w) = self.current_workspace().focused {
            let s = self.current_scr;
            let k = self.current_screen().current_wk;
            self.screens[s].workspaces[k].windows.rotate_left(1);
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
            );
            self.screens[s].workspaces[k].focused = if cur_w == 0 {
                Some(self.current_workspace().windows.len() - 1)
            } else {
                Some(cur_w - 1)
            };
        }
    }

    /// Rotate the current workspace stack down.
    pub fn rotate_windows_down(&mut self) {
        if let Some(cur_w) = self.current_workspace().focused {
            let s = self.current_scr;
            let k = self.current_screen().current_wk;
            self.screens[s].workspaces[k].windows.rotate_right(1);
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
            );
            self.screens[s].workspaces[k].focused =
                if cur_w == self.current_workspace().windows.len() - 1 {
                    Some(0)
                } else {
                    Some(cur_w + 1)
                };
        }
    }

    /// Swaps the current window with the next slave window.
    pub fn swap_with_next_slave(&mut self) {
        if let Some(cur_w) = self.current_workspace().focused {
            if cur_w == 0 {
                return;
            }

            let s = self.current_scr;
            let k = self.current_screen().current_wk;

            let next_w = if cur_w == self.current_workspace().windows.len() - 1 {
                1
            } else {
                cur_w + 1
            };

            let tmp = self.current_workspace().windows[cur_w];
            self.screens[s].workspaces[k].windows[cur_w] =
                self.screens[s].workspaces[k].windows[next_w];
            self.screens[s].workspaces[k].windows[next_w] = tmp;

            self.screens[s].workspaces[k].focused = Some(next_w);

            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
            );
        }
    }

    /// Swaps the current window with the previous slave window.
    pub fn swap_with_prev_slave(&mut self) {
        if let Some(cur_w) = self.current_workspace().focused {
            if cur_w == 0 {
                return;
            }

            let s = self.current_scr;
            let k = self.current_screen().current_wk;

            let prev_w = if cur_w == 1 {
                self.current_workspace().windows.len() - 1
            } else {
                cur_w - 1
            };

            let tmp = self.current_workspace().windows[cur_w];
            self.screens[s].workspaces[k].windows[cur_w] =
                self.screens[s].workspaces[k].windows[prev_w];
            self.screens[s].workspaces[k].windows[prev_w] = tmp;

            self.screens[s].workspaces[k].focused = Some(prev_w);

            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
            );
        }
    }

    /// Changes current window with the master window, or changes the master
    /// window with the first slave window.
    pub fn change_master(&mut self) {
        if self.current_workspace().windows.len() < 2 {
            return;
        }

        if let Some(cur_w) = self.current_workspace().focused {
            let s = self.current_scr;
            let k = self.current_screen().current_wk;

            let other_w = if cur_w == 0 { 1 } else { 0 };

            let tmp = self.current_workspace().windows[cur_w];
            self.screens[s].workspaces[k].windows[cur_w] =
                self.screens[s].workspaces[k].windows[other_w];
            self.screens[s].workspaces[k].windows[other_w] = tmp;
            self.screens[s].workspaces[k].focused = Some(other_w);
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
            );
        }
    }

    /// Runs a system command. Arguments must be separated by spaces.
    /// Note that it DOES NOT runs it inside a shell.
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

    /// Terminate the window manager process.
    pub fn quit() {
        process::exit(0);
    }
}
