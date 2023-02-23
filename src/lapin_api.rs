//! This module defines a bunch of useful public functions to the `Lapin`
//! struct. Check then on docs for `lapin::Lapin`.
use crate::config::Config;
use crate::keys::{match_mods, KeybindSet};
use crate::screens::Screen;
use crate::{Atoms, Lapin};
use std::process;
use xcb::x;
use xcb::xinerama;
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
        let root = x_connection
            .get_setup()
            .roots()
            .next()
            .expect("Failed to retrive root window!")
            .root();
        let atoms = Atoms::intern_all(&x_connection).expect("Cannot init atoms!");

        Lapin {
            x_connection,
            config,
            screens,
            current_scr: current_scr as usize,
            keybinds,
            root,
            atoms,
        }
    }

    /// The last function that should be called, because it'll start the main
    /// loop and bind keys, efectively never returning.
    pub fn init(&mut self, keybinds: &mut KeybindSet) {
        // bind keys.
        for ((modmask, _, code), _) in keybinds.iter() {
            self.x_connection.send_request(&x::GrabKey {
                owner_events: true,
                grab_window: self.root,
                modifiers: *modmask,
                key: *code,
                pointer_mode: x::GrabMode::Async,
                keyboard_mode: x::GrabMode::Async,
            });
        }

        // grab mouse
        self.x_connection.send_request(&x::GrabButton {
            owner_events: true,
            grab_window: self.root,
            event_mask: x::EventMask::BUTTON_MOTION
                | x::EventMask::BUTTON_PRESS
                | x::EventMask::BUTTON_RELEASE,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: x::WINDOW_NONE,
            cursor: x::CURSOR_NONE,
            button: x::ButtonIndex::Any,
            modifiers: match_mods(self.config.mouse_mod).0,
        });

        // register events
        let event_mask = x::EventMask::SUBSTRUCTURE_NOTIFY
            | x::EventMask::STRUCTURE_NOTIFY
            | x::EventMask::SUBSTRUCTURE_REDIRECT
            | x::EventMask::PROPERTY_CHANGE;

        self.x_connection.send_request(&x::ChangeWindowAttributes {
            window: self.root,
            value_list: &[x::Cw::EventMask(event_mask)],
        });

        // setup monitors
        let cookie = self.x_connection.send_request(&xinerama::QueryScreens {});
        let reply = self
            .x_connection
            .wait_for_reply(cookie)
            .expect("Failed to get monitors info");
        for screen in reply.screen_info() {
            self.screens.push(Screen::new(
                &self,
                screen.width,
                screen.height,
                screen.x_org,
                screen.y_org,
            ));
        }

        self.x_connection.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: self.root,
            time: x::CURRENT_TIME,
        });

        self.main_event_loop(keybinds);
    }

    /// Returns the id of the currently focused window.
    pub fn get_focused_window(&self) -> Option<x::Window> {
        if let Some(w) = self.current_workspace().focused {
            if self.current_workspace().ool_focus {
                Some(self.current_workspace().ool_windows[w])
            } else {
                Some(self.current_workspace().windows[w])
            }
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
        for window in &self.current_workspace().windows {
            self.x_connection
                .send_request(&x::UnmapWindow { window: *window });
        }
        for window in &self.current_workspace().ool_windows {
            self.x_connection
                .send_request(&x::UnmapWindow { window: *window });
        }
        self.x_connection.flush().ok();
        self.current_screen_mut().current_wk = wk;
        for window in &self.current_workspace().windows {
            self.x_connection
                .send_request(&x::MapWindow { window: *window });
        }
        for window in &self.current_workspace().ool_windows {
            self.x_connection
                .send_request(&x::MapWindow { window: *window });
        }
        self.x_connection.flush().ok();
        if let Some(focus) = self.current_workspace().focused {
            self.x_connection.send_request(&x::SetInputFocus {
                revert_to: x::InputFocus::PointerRoot,
                focus: self.current_workspace().windows[focus],
                time: x::CURRENT_TIME,
            });
        } else {
            self.x_connection.send_request(&x::SetInputFocus {
                revert_to: x::InputFocus::PointerRoot,
                focus: self.root,
                time: x::CURRENT_TIME,
            });
        }
        self.current_layout().reload(
            &mut self.workspace_windows(),
            &self.x_connection,
            self.current_screen().width,
            self.current_screen().height,
            self.current_screen().x,
            self.current_screen().y,
        );
        self.x_connection.flush().ok();
    }

    /// Rotate the current workspace stack up.
    pub fn rotate_windows_up(&mut self) {
        if self.current_workspace().ool_focus {
            return;
        }
        if let Some(cur_w) = self.current_workspace().focused {
            self.current_workspace_mut().windows.rotate_left(1);
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y,
            );
            self.current_workspace_mut().focused = if cur_w == 0 {
                Some(self.current_workspace().windows.len() - 1)
            } else {
                Some(cur_w - 1)
            };
        }
    }

    /// Rotate the current workspace stack down.
    pub fn rotate_windows_down(&mut self) {
        if self.current_workspace().ool_focus {
            return;
        }
        if let Some(cur_w) = self.current_workspace().focused {
            self.current_workspace_mut().windows.rotate_right(1);
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y,
            );
            self.current_workspace_mut().focused =
                if cur_w == self.current_workspace().windows.len() - 1 {
                    Some(0)
                } else {
                    Some(cur_w + 1)
                };
        }
    }

    /// Swaps the current window with the next slave window.
    pub fn swap_with_next_slave(&mut self) {
        if self.current_workspace().ool_focus {
            return;
        }
        if let Some(cur_w) = self.current_workspace().focused {
            if cur_w == 0 {
                return;
            }

            let next_w = if cur_w == self.current_workspace().windows.len() - 1 {
                1
            } else {
                cur_w + 1
            };

            let tmp = self.current_workspace().windows[cur_w];
            self.current_workspace_mut().windows[cur_w] = self.current_workspace().windows[next_w];
            self.current_workspace_mut().windows[next_w] = tmp;

            self.current_workspace_mut().focused = Some(next_w);

            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y,
            );
        }
    }

    /// Swaps the current window with the previous slave window.
    pub fn swap_with_prev_slave(&mut self) {
        if self.current_workspace().ool_focus {
            return;
        }
        if let Some(cur_w) = self.current_workspace().focused {
            if cur_w == 0 {
                return;
            }

            let prev_w = if cur_w == 1 {
                self.current_workspace().windows.len() - 1
            } else {
                cur_w - 1
            };

            let tmp = self.current_workspace().windows[cur_w];
            self.current_workspace_mut().windows[cur_w] = self.current_workspace().windows[prev_w];
            self.current_workspace_mut().windows[prev_w] = tmp;

            self.current_workspace_mut().focused = Some(prev_w);

            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y,
            );
        }
    }

    /// Changes current window with the master window, or changes the master
    /// window with the first slave window.
    pub fn change_master(&mut self) {
        if self.current_workspace().ool_focus {
            return;
        }
        if self.current_workspace().windows.len() < 2 {
            return;
        }

        if let Some(cur_w) = self.current_workspace().focused {
            let other_w = if cur_w == 0 { 1 } else { 0 };

            let tmp = self.current_workspace().windows[cur_w];
            self.current_workspace_mut().windows[cur_w] = self.current_workspace().windows[other_w];
            self.current_workspace_mut().windows[other_w] = tmp;
            self.current_workspace_mut().focused = Some(other_w);

            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y,
            );
        }
    }

    pub fn toggle_ool(&mut self) {
        if let Some(w) = self.current_workspace().focused {
            if self.current_workspace().ool_focus {
                let window = self.current_workspace_mut().ool_windows.remove(w);
                self.current_workspace_mut().windows.insert(0, window);
                self.current_workspace_mut().ool_focus = false;
                self.current_workspace_mut().focused = Some(0);
                self.current_layout().newwin(
                    &mut self.workspace_windows(),
                    &self.x_connection,
                    self.current_screen().width,
                    self.current_screen().height,
                    self.current_screen().x,
                    self.current_screen().y,
                );
            } else {
                let window = self.current_workspace_mut().windows.remove(w);
                self.current_workspace_mut().ool_windows.insert(0, window);
                self.current_workspace_mut().ool_focus = true;
                self.current_workspace_mut().focused = Some(0);
                self.current_layout().delwin(
                    &mut self.workspace_windows(),
                    self.current_workspace().focused,
                    &self.x_connection,
                    self.current_screen().width,
                    self.current_screen().height,
                    self.current_screen().x,
                    self.current_screen().y,
                );
            }
        }
    }

    /// Changes the focus to the next monitor.
    pub fn next_screen(&mut self) {
        self.change_screen(false);
    }

    /// Changes the focus to the previous monitor.
    pub fn prev_screen(&mut self) {
        self.change_screen(true);
    }

    /// Sends the focused window to other workspace.
    pub fn send_window_to_workspace(&mut self, workspace: usize) {
        if self.current_screen().current_wk == workspace {
            return;
        }

        if let Some(w) = self.current_workspace().focused {
            let (ool, window) = if self.current_workspace().ool_focus {
                let window = self.current_workspace_mut().ool_windows.remove(w);
                self.current_screen_mut().workspaces[workspace].ool_windows.insert(0, window);
                self.current_screen_mut().workspaces[workspace].ool_focus = true;
                (true, window)
            } else {
                let window = self.current_workspace_mut().windows.remove(w);
                self.current_screen_mut().workspaces[workspace].windows.insert(0, window);
                self.current_screen_mut().workspaces[workspace].ool_focus = false;
                (false, window)
            };
            self.current_screen_mut().workspaces[workspace].focused = Some(0);

            self.x_connection.send_request(&x::UnmapWindow {
                window,
            });
            self.x_connection.flush().ok();

            self.reset_focus_after_removing(
                self.current_scr,
                self.current_screen().current_wk,
                self.current_workspace().focused.unwrap(),
                ool,
            );
            self.x_connection.flush().ok();

            if !ool {
                self.current_layout().delwin(
                    &mut self.workspace_windows(),
                    self.current_workspace().focused,
                    &self.x_connection,
                    self.current_screen().width,
                    self.current_screen().height,
                    self.current_screen().x,
                    self.current_screen().y,
                );
            }
            self.x_connection.flush().ok();
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
