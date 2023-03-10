//! This module defines a bunch of useful public functions to the `Lapin`
//! struct. Check then on docs for `Lapin`.
use crate::config::Config;
use crate::keys::{match_mods, Callback, KeybindSet};
use crate::screens::Screen;
use crate::{Atoms, Lapin};
use std::process;
use xcb::x;
use xcb::xinerama;
use xcb::Connection;
use xcb::Xid;

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
    ///
    /// The last parameter is a callback to be called right before the
    /// event loop starts, after everything is already set up. As with
    /// keybinds, you can use the macro `lazy!` to create it.
    pub fn init(&mut self, keybinds: &mut KeybindSet, callback: Option<&mut Callback>) {
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

        //
        // setup atoms for EWMH and stuff.
        //

        // check window
        let window: x::Window = self.x_connection.generate_id();
        self.x_connection.send_request(&x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: self.root,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: self
                .x_connection
                .get_setup()
                .roots()
                .next()
                .unwrap()
                .root_visual(),
            value_list: &[],
        });
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: self.root,
            property: self.atoms.net_supporting_wm_check,
            r#type: x::ATOM_WINDOW,
            data: &[window],
        });

        // client list
        self.x_connection
            .send_request(&x::ChangeProperty::<x::Window> {
                mode: x::PropMode::Append,
                window: self.root,
                property: self.atoms.net_client_list,
                r#type: x::ATOM_WINDOW,
                data: &[],
            });

        // desktops number and names
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: self.root,
            property: self.atoms.net_number_of_desktops,
            r#type: x::ATOM_CARDINAL,
            data: &[self.config.workspaces.len() as u32],
        });
        // transform the array of &'static str in a vec of u8
        let mut c_str_vec: Vec<u8> = Vec::new();
        for wk in self.config.workspaces {
            for ch in wk.as_bytes() {
                c_str_vec.push(*ch);
            }
            c_str_vec.push('\0' as u8);
        }
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: self.root,
            property: self.atoms.net_desktop_names,
            r#type: x::ATOM_STRING,
            data: &c_str_vec[..],
        });

        // desktop viewport
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: self.root,
            property: self.atoms.net_desktop_viewport,
            r#type: x::ATOM_CARDINAL,
            data: &[0 as u32, 0 as u32],
        });

        // WM name
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: window,
            property: self.atoms.net_wm_name,
            r#type: x::ATOM_STRING,
            data: b"Le Petit Lapin",
        });

        // set the supported atoms
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: self.root,
            property: self.atoms.net_supported,
            r#type: x::ATOM_ATOM,
            data: &[
                self.atoms.net_supported,
                self.atoms.net_client_list,
                self.atoms.net_number_of_desktops,
                self.atoms.net_current_desktop,
                self.atoms.net_supporting_wm_check,
                self.atoms.net_desktop_viewport,
                self.atoms.net_wm_name,
                self.atoms.net_wm_desktop,
                self.atoms.net_wm_state,
                self.atoms.net_wm_state_fullscreen,
                self.atoms.net_wm_action_fullscreen,
            ],
        });

        // if has a callback, calls it
        if let Some(callback) = callback {
            callback(self);
        }

        // current desktop is set after the callback so the user can
        // change it without pain if wishes.
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: self.root,
            property: self.atoms.net_current_desktop,
            r#type: x::ATOM_CARDINAL,
            data: &[self.current_screen().current_wk as u32],
        });

        self.x_connection.flush().ok();

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

    /// Kills the currently focused client.
    pub fn killfocused(&mut self) {
        let Some(window) = self.get_focused_window() else { return };
        self.x_connection.send_request(&x::KillClient {
            resource: window.resource_id(),
        });
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
        if self.current_screen().current_wk == wk {
            return;
        }
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
        // change the property for the sake of ewmh
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_current_desktop,
            r#type: x::ATOM_CARDINAL,
            data: &[self.current_screen().current_wk as u32],
        });
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
            let focused = if self.current_workspace().ool_focus {
                self.current_workspace().ool_windows[focus]
            } else {
                self.current_workspace().windows[focus]
            };
            self.x_connection.send_request(&x::SetInputFocus {
                revert_to: x::InputFocus::PointerRoot,
                focus: focused,
                time: x::CURRENT_TIME,
            });
        } else {
            self.x_connection.send_request(&x::SetInputFocus {
                revert_to: x::InputFocus::PointerRoot,
                focus: self.root,
                time: x::CURRENT_TIME,
            });
        }
        let (width, height, x, y) = self.calculate_layout_coordinates();
        self.current_layout().reload(
            &mut self.workspace_windows(),
            &self.x_connection,
            width,
            height,
            x,
            y,
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
            let (width, height, x, y) = self.calculate_layout_coordinates();
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                width,
                height,
                x,
                y,
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
            let (width, height, x, y) = self.calculate_layout_coordinates();
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                width,
                height,
                x,
                y,
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

            let (width, height, x, y) = self.calculate_layout_coordinates();
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                width,
                height,
                x,
                y,
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

            let (width, height, x, y) = self.calculate_layout_coordinates();
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                width,
                height,
                x,
                y,
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

            let (width, height, x, y) = self.calculate_layout_coordinates();
            self.current_layout().reload(
                &mut self.workspace_windows(),
                &self.x_connection,
                width,
                height,
                x,
                y,
            );
        }
    }

    /// Toggles the reserved space in the current workspace.
    pub fn toggle_reserved_space(&mut self) {
        self.current_workspace_mut().respect_reserved_space =
            !self.current_workspace().respect_reserved_space;
        let (width, height, x, y) = self.calculate_layout_coordinates();
        self.current_layout().reload(
            &mut self.workspace_windows(),
            &self.x_connection,
            width,
            height,
            x,
            y,
        );
        self.x_connection.flush().ok();
    }

    /// Sends window to the "out of layout" stack, or sends it back to
    /// the main stack. Out of layout windows are not managed by the
    /// layout, so they stay floating.
    pub fn toggle_ool(&mut self) {
        if let Some(w) = self.current_workspace().focused {
            if self.current_workspace().ool_focus {
                let window = self.current_workspace_mut().ool_windows.remove(w);
                self.current_workspace_mut().windows.insert(0, window);
                self.current_workspace_mut().ool_focus = false;
                self.current_workspace_mut().focused = Some(0);
                let (width, height, x, y) = self.calculate_layout_coordinates();
                self.current_layout().newwin(
                    &mut self.workspace_windows(),
                    &self.x_connection,
                    width,
                    height,
                    x,
                    y,
                );
                self.x_connection.send_request(&x::ConfigureWindow {
                    window,
                    value_list: &[x::ConfigWindow::BorderWidth(
                        self.current_layout().border_width() as u32,
                    )],
                });
            } else {
                let window = self.current_workspace_mut().windows.remove(w);
                self.current_workspace_mut().ool_windows.insert(0, window);
                self.current_workspace_mut().ool_focus = true;
                self.current_workspace_mut().focused = Some(0);
                let (width, height, x, y) = self.calculate_layout_coordinates();
                self.current_layout().delwin(
                    &mut self.workspace_windows(),
                    self.current_workspace().focused,
                    &self.x_connection,
                    width,
                    height,
                    x,
                    y,
                );
                self.x_connection.send_request(&x::ConfigureWindow {
                    window,
                    value_list: &[x::ConfigWindow::BorderWidth(
                        self.config.border_width as u32,
                    )],
                });
                self.x_connection
                    .send_request(&x::ChangeProperty::<x::Atom> {
                        mode: x::PropMode::Replace,
                        window,
                        property: self.atoms.net_wm_state,
                        r#type: x::ATOM_ATOM,
                        data: &[],
                    });
            }
            self.x_connection.flush().ok();
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
                self.current_screen_mut().workspaces[workspace]
                    .ool_windows
                    .insert(0, window);
                self.current_screen_mut().workspaces[workspace].ool_focus = true;
                (true, window)
            } else {
                let window = self.current_workspace_mut().windows.remove(w);
                self.current_screen_mut().workspaces[workspace]
                    .windows
                    .insert(0, window);
                self.current_screen_mut().workspaces[workspace].ool_focus = false;
                (false, window)
            };
            self.current_screen_mut().workspaces[workspace].focused = Some(0);

            // change the window desktop for EWMH
            self.x_connection.send_request(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window,
                property: self.atoms.net_wm_desktop,
                r#type: x::ATOM_CARDINAL,
                data: &[workspace as u32],
            });
            // unmaps window
            self.x_connection.send_request(&x::UnmapWindow { window });
            self.x_connection.flush().ok();

            self.reset_focus_after_removing(
                self.current_scr,
                self.current_screen().current_wk,
                self.current_workspace().focused.unwrap(),
                ool,
            );
            self.x_connection.flush().ok();

            if !ool {
                let (width, height, x, y) = self.calculate_layout_coordinates();
                self.current_layout().delwin(
                    &mut self.workspace_windows(),
                    self.current_workspace().focused,
                    &self.x_connection,
                    width,
                    height,
                    x,
                    y,
                );
            }
            self.x_connection.flush().ok();
        }
    }

    /// Sends the focused window to the next screen (monitor).
    pub fn send_window_to_next_screen(&mut self) {
        if self.screens.len() >= 2 {
            self.change_window_screen(false);
        }
    }

    /// Sends the focused window to the previous screen (monitor).
    pub fn send_window_to_prev_screen(&mut self) {
        if self.screens.len() >= 2 {
            self.change_window_screen(true);
        }
    }

    /// Fullscreens a window. Kind of a hack, just toggles ool, sets x and y to the monitor
    /// location and removes the border.
    pub fn fullscreen(&mut self) {
        if let Some(window) = self.get_focused_window() {
            if !self.current_workspace().ool_focus {
                self.toggle_ool();
            }
            let list = [
                x::ConfigWindow::X(self.current_screen().x as i32),
                x::ConfigWindow::Y(self.current_screen().y as i32),
                x::ConfigWindow::Width(self.current_screen().width as u32),
                x::ConfigWindow::Height(self.current_screen().height as u32),
                x::ConfigWindow::BorderWidth(0),
                x::ConfigWindow::StackMode(x::StackMode::Above),
            ];
            self.x_connection.send_request(&x::ConfigureWindow {
                window,
                value_list: &list,
            });
            self.x_connection.send_request(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window: window,
                property: self.atoms.net_wm_state,
                r#type: x::ATOM_ATOM,
                data: &[self.atoms.net_wm_state_fullscreen],
            });
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
