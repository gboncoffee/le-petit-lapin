//! Copyright (c) 2023 Gabriel G. de Brito
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//!
//! # Le Petit Lapin - The cute X window manager.
//!
//! Le Petit Lapin is a X window manager written in Rust as a library. One must
//! create a binary Cargo crate that depends on `lapin` to build it with a desired
//! configuration.
//!
//! The name "Le petit lapin" was choosen by a friend of mine and it means "The
//! little bunny" in French, but I'm not 100% sure about that because I don't speak
//! French.
//!
//! ## Quickstart
//!
//! To use Lapin is to write your own window manager in Rust, depending on this
//! crate. A sample config would look like this:
//! ```no_run
//! use lapin::keys::*;
//! use lapin::layouts::*;
//! use lapin::*;
//!
//! fn main() {
//!     // The first thing to do is always call Lapin::connect() to create a
//!     // new Lapin object and connect to the X server.
//!     let mut lapin = Lapin::connect();
//!
//!     // A good pratice is to define things you'll use a lot later as const.
//!     const MODKEY: &str = "Meta";
//!     const TERMINAL: &str = "alacritty";
//!
//!     // Some configurations are kept inside a Config struct, member of
//!     // Lapin. "mouse_mod", for example, is the modifier we use with the
//!     // mouse buttons to move windows around (with button 1) and resize then
//!     // (with button 2).
//!     lapin.config.mouse_mod = &[MODKEY];
//!     // The workspaces number and name are handled here too. By default,
//!     // they're 9 workspaces named as numbers from 1 to 9. In this example,
//!     // we'll use 3:
//!     lapin.config.workspaces = &["dev", "web", "sys"];
//!
//!     // Border colors are also handled here. They're u32 numbers in the form
//!     // ARGB.
//!                                      // A R G B
//!     lapin.config.border_color       = 0xff000000;
//!     lapin.config.border_color_focus = 0xffffffff;
//!
//!     // Keybinds are handled by the main_loop function, separately from the
//!     // window manager struct. We create a new empty set then bind some keys
//!     // to it:
//!     let mut keybinds = KeybindSet::new();
//!     keybinds.bindall(vec![
//!         // workspace keys
//!         (&[MODKEY], "1", lazy! {wm, wm.goto_workspace(1)}),
//!         (&[MODKEY], "2", lazy! {wm, wm.goto_workspace(2)}),
//!         (&[MODKEY], "3", lazy! {wm, wm.goto_workspace(3)}),
//!         // quit
//!         (&[MODKEY], "q", lazy! {Lapin::quit()}),
//!         // spawns
//!         (&[MODKEY], "Return", lazy! {Lapin::spawn(TERMINAL)}),
//!         (&[MODKEY], "n", lazy! {Lapin::spawn("chromium")}),
//!         (&[MODKEY], "a", lazy! {Lapin::spawn("rofi -show run")}),
//!         // kill focus
//!         (&[MODKEY], "w", lazy! {wm, wm.killfocused()}),
//!         // change focus
//!         (&[MODKEY], "j", lazy! {wm, wm.nextwin()}),
//!         (&[MODKEY], "k", lazy! {wm, wm.prevwin()}),
//!         // change layout
//!         (&[MODKEY], "space", lazy! {wm, wm.next_layout()}),
//!         (&[MODKEY, "Shift"], "space", lazy! {wm, wm.prev_layout()}),
//!
//!         //
//!         // You can check other useful functions to use as keybinds in the
//!         // docs for the Lapin struct.
//!         //
//!     ]);
//!
//!     // You can create Layouts with different configs. To use default ones,
//!     // use their "new()" functions. Custom layouts can be created just by
//!     // implementing the Layout trait for them. The default config has
//!     // the three default layouts with default config.
//!     let tile = Tiling {
//!         name: "tile",
//!         borders: 4,
//!         master_factor: 1.0 / 2.0,
//!         gaps: 4,
//!     };
//!     let max = Maximized::new();
//!     let float = Floating {
//!         name: "float",
//!         borders: 4,
//!     };
//!
//!     // Use the layouts! macro to create a list of layouts.
//!     lapin.config.layouts = layouts![tile, max, float];
//!
//!     // With a callback, you can easily access and modify the WM
//!     // and autostart programs as soon as everything is set up.
//!     let mut callback = lazy![wm, {
//!        let home = env!("HOME");
//!	   Lapin::spawn("picom");
//!        Lapin::spawn(&format!(
//!            "feh --no-fehbg --bg-fill {home}/.config/wallpaper"
//!        ));
//!        println!("There's {} monitors available", wm.screens.len());
//!     }];
//!
//!     // The last thing to do is starting the window manager with the keybind
//!     // set with Lapin::init(&mut KeybindSet, Option<&mut Callback>).
//!     lapin.init(&mut keybinds, Some(&mut callback));
//! }
//! ```

pub mod config;
pub mod keys;
pub mod lapin_api;
pub mod layouts;
pub mod rules;
pub mod screens;
pub mod utils;

use config::*;
use keys::*;
use rules::*;
use screens::*;
use std::time;
use xcb::x;
use xcb::Connection;
use xcb::Xid;

#[rustfmt::skip]
xcb::atoms_struct! {
    /// Atoms struct for the window manager.
    pub struct Atoms {
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
	pub net_number_of_desktops => b"_NET_NUMBER_OF_DESKTOPS" only_if_exists = false,
	pub net_desktop_geometry => b"_NET_DESKTOP_GEOMETRY" only_if_exists = false,
	pub net_desktop_viewport => b"_NET_DESKTOP_VIEWPORT" only_if_exists = false,
	pub net_current_desktop => b"_NET_CURRENT_DESKTOP" only_if_exists = false,
	pub net_desktop_names => b"_NET_DESKTOP_NAMES" only_if_exists = false,
	pub net_workarea => b"_NET_WORKAREA" only_if_exists = false,
	pub net_supporting_wm_check => b"_NET_SUPPORTING_WM_CHECK" only_if_exists = false,
    }
}

/// The window manager I suppose.
pub struct Lapin {
    /// The connection with the X server via the XCB crate. Only touch
    /// it if you know what you're doing.
    pub x_connection: Connection,
    /// Configuration for the WM.
    pub config: Config,
    /// Keybinds for the WM.
    pub keybinds: KeybindSet,
    /// Screens (monitors). Automatically set by Xinerama on
    /// startup. Don't touch them.
    pub screens: Vec<Screen>,
    /// Atoms. Only touch them if you know what you're doing.
    pub atoms: Atoms,
    current_scr: usize,
    root: x::Window,
}

impl Lapin {
    /// Returns the window location as (screen, workspace, index, is out of the layout?)
    fn window_location(&self, win: x::Window) -> Option<(usize, usize, usize, bool)> {
        for (s, screen) in self.screens.iter().enumerate() {
            for (k, workspace) in screen.workspaces.iter().enumerate() {
                for (w, window) in workspace.windows.iter().enumerate() {
                    if *window == win {
                        return Some((s, k, w, false));
                    }
                }
                for (w, window) in workspace.ool_windows.iter().enumerate() {
                    if *window == win {
                        return Some((s, k, w, true));
                    }
                }
            }
        }
        None
    }

    fn add_border(&self, w: x::Window) {
        self.x_connection.send_request(&x::ConfigureWindow {
            window: w,
            value_list: &[x::ConfigWindow::BorderWidth(
                self.current_layout().border_width() as u32,
            )],
        });
    }

    fn color_focused_border(&self, w: x::Window) {
        self.x_connection.send_request(&x::ChangeWindowAttributes {
            window: w,
            value_list: &[x::Cw::BorderPixel(self.config.border_color_focus)],
        });
    }

    fn restore_border(&self, window: x::Window) {
        self.x_connection.send_request(&x::ChangeWindowAttributes {
            window,
            value_list: &[x::Cw::BorderPixel(self.config.border_color)],
        });
    }

    fn add_client_to_atom(&self, window: x::Window) {
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Append,
            window: self.root,
            property: self.atoms.net_client_list,
            r#type: x::ATOM_WINDOW,
            data: &window.resource_id().to_ne_bytes(),
        });
    }

    /// Apply rules for a window, returns what must be done with it (add_border, ool, workspace).
    fn apply_rules(&self, window: x::Window) -> (bool, bool, usize) {
        let mut add_border = true;
        let mut ool = false;
        let mut workspace = self.current_screen().current_wk;

        let (class1, class2) = if let Some(t) = self.get_class(window) {
            t
        } else {
            return (add_border, ool, workspace);
        };

        for rule in self.config.rules.iter() {
            if rule.property == Property::Class(class1.clone())
                || rule.property == Property::Class(class2.clone())
            {
                match rule.apply {
                    Apply::Workspace(n) => workspace = n,
                    Apply::Float => ool = true,
                    Apply::Fullscreen => {
                        self.x_connection.send_request(&x::ConfigureWindow {
                            window,
                            value_list: &[
                                x::ConfigWindow::X(self.current_screen().x as i32),
                                x::ConfigWindow::Y(self.current_screen().y as i32),
                                x::ConfigWindow::Width(self.current_screen().width as u32),
                                x::ConfigWindow::Height(self.current_screen().height as u32),
                            ],
                        });
                        self.x_connection.flush().ok();
                        ool = true;
                        add_border = false;
                    }
                }
            }
        }

        return (add_border, ool, workspace);
    }

    fn manage_window(&mut self, ev: x::MapRequestEvent) {
        // check if we really need to manage the window
        if self.window_location(ev.window()).is_some() {
            return;
        }
        let cookie = self.x_connection.send_request(&x::GetWindowAttributes {
            window: ev.window(),
        });
        let reply = self.x_connection.wait_for_reply(cookie);
        if let Ok(reply) = reply {
            if reply.override_redirect() {
                return;
            }
        } else {
            return;
        }

        // add required attributes
        self.x_connection.send_request(&x::ChangeWindowAttributes {
            window: ev.window(),
            value_list: &[
                x::Cw::BorderPixel(self.config.border_color),
                x::Cw::EventMask(
                    x::EventMask::ENTER_WINDOW
                        | x::EventMask::PROPERTY_CHANGE
                        | x::EventMask::STRUCTURE_NOTIFY,
                ),
            ],
        });

        let (add_border, ool, workspace) = self.apply_rules(ev.window());

        if add_border {
            self.add_border(ev.window());
        }
        if let Some(old_win) = self.get_focused_window() {
            self.restore_border(old_win);
        }

        if ool {
            self.current_screen_mut().workspaces[workspace]
                .ool_windows
                .insert(0, ev.window());
        } else {
            self.current_screen_mut().workspaces[workspace]
                .windows
                .insert(0, ev.window());
        }

        if workspace == self.current_screen().current_wk {
            self.current_layout().newwin(
                &mut self.workspace_windows(),
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y,
            );
            self.x_connection.send_request(&x::MapWindow {
                window: ev.window(),
            });
            self.set_focus(
                ev.window(),
                self.current_scr,
                self.current_screen().current_wk,
                0,
                ool,
                true,
            );
        }
        self.add_client_to_atom(ev.window());

        self.x_connection.flush().ok();
    }

    fn reset_focus_after_removing(&mut self, s: usize, k: usize, w: usize, ool: bool) {
        let ool = if ool && self.current_workspace().ool_windows.len() > 0 {
            true
        } else if self.current_workspace().windows.len() > 0 {
            false
        } else if self.current_workspace().ool_windows.len() > 0 {
            true
        } else {
            self.current_workspace_mut().focused = None;
            return;
        };

        let compare = if ool {
            self.current_workspace().ool_windows.len()
        } else {
            self.current_workspace().windows.len()
        };
        let w = if w >= compare { compare - 1 } else { w };
        let window = if ool {
            self.current_workspace().ool_windows[w]
        } else {
            self.current_workspace().windows[w]
        };
        self.set_focus(window, s, k, w, ool, true);
    }

    fn unmanage_window(&mut self, window: x::Window, set_focus: bool) {
        if let Some((s, k, w, ool)) = self.window_location(window) {
            if ool {
                self.current_workspace_mut().ool_windows.remove(w);
            } else {
                self.current_workspace_mut().windows.remove(w);
            }
            self.x_connection.send_request(&x::ChangeProperty::<u8> {
                mode: x::PropMode::Replace,
                window: self.root,
                property: self.atoms.net_client_list,
                r#type: x::ATOM_WINDOW,
                data: &[],
            });
            self.x_connection.flush().ok();
            for scr in &self.screens {
                for wk in &scr.workspaces {
                    for window in &wk.windows {
                        self.add_client_to_atom(*window);
                    }
                    for window in &wk.ool_windows {
                        self.add_client_to_atom(*window);
                    }
                }
            }
            if set_focus {
                self.reset_focus_after_removing(s, k, w, ool);
            }
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
            } else if !self.current_workspace().ool_focus {
                if let Some(number) = self.current_workspace().focused {
                    self.current_layout().changewin(
                        &mut self.workspace_windows(),
                        number,
                        &self.x_connection,
                        self.current_screen().width,
                        self.current_screen().height,
                        self.current_screen().x,
                        self.current_screen().y,
                    );
                }
            }
            self.x_connection.flush().ok();
        }
    }

    fn set_focus(
        &mut self,
        window: x::Window,
        s: usize,
        k: usize,
        w: usize,
        ool: bool,
        raise: bool,
    ) {
        self.current_scr = s;
        self.current_screen_mut().current_wk = k;
        self.current_workspace_mut().focused = Some(w);
        self.current_workspace_mut().ool_focus = ool;
        self.x_connection.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        });
        if raise {
            self.x_connection.send_request(&x::ConfigureWindow {
                window,
                value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
            });
        }
        self.color_focused_border(window);
        self.x_connection.flush().ok();
    }

    fn toggle_focus(&mut self, window: x::Window, raise: bool) {
        if let Some((s, k, w, ool)) = self.window_location(window) {
            if let Some(window) = self.get_focused_window() {
                self.restore_border(window);
            }
            self.set_focus(window, s, k, w, ool, raise);
        }
    }

    fn init_mouse_action(
        &mut self,
        event: &x::ButtonPressEvent,
    ) -> (
        Option<i16>,
        Option<i16>,
        Option<i16>,
        Option<i16>,
        Option<x::Window>,
    ) {
        let cookie = self.x_connection.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(event.child()),
        });
        let reply = if let Ok(res) = self.x_connection.wait_for_reply(cookie) {
            res
        } else {
            return (None, None, None, None, None);
        };
        let (x, y) = (reply.x(), reply.y());

        (
            Some(event.root_x() - x),
            Some(event.root_y() - y),
            Some(reply.x()),
            Some(reply.y()),
            Some(event.child()),
        )
    }

    fn handle_motion(
        &self,
        ev: x::MotionNotifyEvent,
        x_diff: i16,
        y_diff: i16,
        x_pos: i16,
        y_pos: i16,
        window: x::Window,
    ) {
        if ev.state().contains(x::KeyButMask::BUTTON1) {
            let list = [
                x::ConfigWindow::X((ev.root_x() - x_diff) as i32),
                x::ConfigWindow::Y((ev.root_y() - y_diff) as i32),
                x::ConfigWindow::BorderWidth(self.config.border_width as u32),
            ];
            self.x_connection.send_request(&x::ConfigureWindow {
                window,
                value_list: &list,
            });
        } else if ev.state().contains(x::KeyButMask::BUTTON3) {
            let list = [
                x::ConfigWindow::Width((ev.root_x() - x_pos) as u32),
                x::ConfigWindow::Height((ev.root_y() - y_pos) as u32),
            ];
            self.x_connection.send_request(&x::ConfigureWindow {
                window,
                value_list: &list,
            });
        }
        self.x_connection.flush().ok();
    }

    fn change_win(&mut self, previous: bool) {
        let s = self.current_scr;
        let k = self.current_screen().current_wk;
        let ool = self.current_workspace().ool_focus;

        if let Some(w) = self.current_workspace().focused {
            let ool_n = self.current_workspace().ool_windows.len();
            let win_n = self.current_workspace().windows.len();
            if win_n + ool_n <= 1 {
                return;
            }

            let w = w as i32;
            let (this, other) = if ool { (ool_n, win_n) } else { (win_n, ool_n) };
            let new_w = if previous { w - 1 } else { w + 1 };
            let (new_w, ool) = if new_w < 0 {
                if other > 0 {
                    ((other - 1), !ool)
                } else {
                    (this - 1, ool)
                }
            } else if new_w >= this as i32 {
                if other > 0 {
                    (0, !ool)
                } else {
                    (0, ool)
                }
            } else {
                (new_w as usize, ool)
            };
            let window = if ool {
                self.current_workspace().ool_windows[new_w]
            } else {
                self.current_workspace().windows[new_w]
            };
            self.restore_border(self.get_focused_window().unwrap());
            self.set_focus(window, s, k, new_w, ool, true);
            self.x_connection.flush().ok();
            if !ool {
                self.current_layout().changewin(
                    &mut self.workspace_windows(),
                    new_w,
                    &self.x_connection,
                    self.current_screen().width,
                    self.current_screen().height,
                    self.current_screen().x,
                    self.current_screen().y,
                );
            }
        }
    }

    fn change_layout(&mut self, previous: bool) {
        let new_n = if previous {
            if self.current_workspace().layout == 0 {
                self.config.layouts.len() - 1
            } else {
                self.current_workspace().layout - 1
            }
        } else {
            self.current_workspace().layout + 1
        };
        let l = if new_n >= self.config.layouts.len() {
            0
        } else {
            new_n
        };

        self.current_workspace_mut().layout = l;

        if let Some(cur_win) = self.get_focused_window() {
            for window in self.workspace_windows() {
                self.add_border(*window);
                if *window == cur_win {
                    self.color_focused_border(*window);
                } else {
                    self.restore_border(*window);
                }
            }
        }

        self.x_connection.flush().ok();

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

    fn change_screen(&mut self, previous: bool) {
        if let Some(old_win) = self.get_focused_window() {
            self.restore_border(old_win);
        }
        let new_s = if previous {
            (self.current_scr as isize) - 1
        } else {
            (self.current_scr as isize) + 1
        };
        let new_s = if new_s < 0 {
            (self.screens.len() - 1) as usize
        } else if new_s >= self.screens.len() as isize {
            0
        } else {
            new_s as usize
        };

        self.current_scr = new_s;
        let window = if let Some(window) = self.get_focused_window() {
            self.color_focused_border(window);
            window
        } else {
            self.root
        };

        self.x_connection.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        });

        // change the current workspace for ewmh
        self.x_connection.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_current_desktop,
            r#type: x::ATOM_CARDINAL,
            data: &[self.current_screen().current_wk as u32],
        });

        self.x_connection.flush().ok();
    }

    fn change_window_screen(&mut self, previous: bool) {
        if let Some(window) = self.get_focused_window() {
            let other_screen = if previous {
                (self.current_scr as isize) - 1
            } else {
                (self.current_scr as isize) + 1
            };
            let other_screen: usize = if other_screen < 0 {
                self.screens.len() - 1
            } else if other_screen >= (self.screens.len() as isize) {
                0
            } else {
                other_screen as usize
            };

            let ool = self.current_workspace().ool_focus;
            let s = self.current_scr;
            let k = self.current_screen().current_wk;
            let w = self.current_workspace().focused.unwrap();

            if ool {
                self.current_workspace_mut().ool_windows.remove(w);
            } else {
                self.current_workspace_mut().windows.remove(w);
            }

            self.reset_focus_after_removing(s, k, w, ool);
            self.restore_border(window);

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

            let other_k = self.screens[other_screen].current_wk;
            self.screens[other_screen].workspaces[other_k].focused = Some(0);
            if ool {
                let list = [
                    x::ConfigWindow::X(self.screens[other_screen].x as i32),
                    x::ConfigWindow::Y(self.screens[other_screen].y as i32),
                    x::ConfigWindow::StackMode(x::StackMode::Above),
                ];
                self.screens[other_screen].workspaces[other_k]
                    .ool_windows
                    .insert(0, window);
                self.x_connection.send_request(&x::ConfigureWindow {
                    window,
                    value_list: &list,
                });
                self.screens[other_screen].workspaces[other_k].ool_focus = true;
            } else {
                let other_layout = self.screens[other_screen].workspaces[other_k].layout;
                self.screens[other_screen].workspaces[other_k]
                    .windows
                    .insert(0, window);
                self.config.layouts[other_layout].newwin(
                    &mut self.screens[other_screen].workspaces[other_k]
                        .windows
                        .iter(),
                    &self.x_connection,
                    self.screens[other_screen].width,
                    self.screens[other_screen].height,
                    self.screens[other_screen].x,
                    self.screens[other_screen].y,
                );
                self.screens[other_screen].workspaces[other_k].ool_focus = false;
            }
            self.x_connection.flush().ok();
        }
    }

    /// The main event loop of the window manager.
    fn main_event_loop(&mut self, keybinds: &mut KeybindSet) -> ! {
        // state for window motions.
        let mut diff_x = None;
        let mut diff_y = None;
        let mut pos_x = None;
        let mut pos_y = None;
        let mut move_window = None;
        // gambiarra to solve the problem of input when mapping windows
        let mut last_map = time::SystemTime::now();
        // gambiarra to solve the problem of the focus after destroying a window over another
        // window
        let mut last_mouse_change_focus = time::SystemTime::now();

        loop {
            match utils::get_x_event(&self.x_connection) {
                x::Event::MapRequest(ev) => {
                    last_map = time::SystemTime::now();
                    self.manage_window(ev);
                }
                x::Event::DestroyNotify(ev) => {
                    last_map = time::SystemTime::now();
                    let set_focus = if time::SystemTime::now()
                        .duration_since(last_mouse_change_focus)
                        .unwrap()
                        > time::Duration::from_millis(100)
                    {
                        true
                    } else {
                        false
                    };
                    self.unmanage_window(ev.window(), set_focus);
                }
                x::Event::EnterNotify(ev) => {
                    if time::SystemTime::now().duration_since(last_map).unwrap()
                        > time::Duration::from_millis(100)
                    {
                        last_mouse_change_focus = time::SystemTime::now();
                        self.toggle_focus(ev.event(), self.config.mouse_raises_window);
                    }
                }
                x::Event::KeyPress(ev) => {
                    if let Some(callback) = keybinds.get_callback(ev.detail(), ev.state()) {
                        callback(self);
                    }
                }
                x::Event::ButtonPress(ev) => {
                    if self.current_layout().allow_motions() || self.current_workspace().ool_focus {
                        (diff_x, diff_y, pos_x, pos_y, move_window) = self.init_mouse_action(&ev)
                    }
                }
                x::Event::ButtonRelease(_) => (diff_x, diff_y) = (None, None),
                x::Event::MotionNotify(ev) => {
                    if self.current_layout().allow_motions() || self.current_workspace().ool_focus {
                        if let Some(x_d) = diff_x {
                            let y_d = diff_y.unwrap();
                            let x_p = pos_x.unwrap();
                            let y_p = pos_y.unwrap();
                            let win = move_window.unwrap();
                            self.handle_motion(ev, x_d, y_d, x_p, y_p, win);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn current_screen<'a>(&'a self) -> &'a Screen {
        &self.screens[self.current_scr]
    }

    fn current_screen_mut<'a>(&'a mut self) -> &'a mut Screen {
        &mut self.screens[self.current_scr]
    }

    fn current_workspace<'a>(&'a self) -> &'a Workspace {
        let wk = self.current_screen().current_wk;
        &self.current_screen().workspaces[wk]
    }

    fn current_workspace_mut<'a>(&'a mut self) -> &'a mut Workspace {
        let wk = self.current_screen().current_wk;
        &mut self.current_screen_mut().workspaces[wk]
    }

    fn current_layout<'a>(&'a self) -> &'a Box<dyn layouts::Layout> {
        let layout = self.current_workspace().layout;
        &self.config.layouts[layout]
    }

    fn workspace_windows<'a>(&'a self) -> std::slice::Iter<'a, x::Window> {
        self.current_workspace().windows.iter()
    }

    /*
     * The following functions are the most terrible code you'll ever see in your fucking life.
     * Their only goal is to actually get the fucking window title and classes. Unfortunatelly, it
     * looks like xcb was designed to don't allow you to do that. Try getting them to work without
     * all these stupid ugly workarounds and you'll see. Good luck.
     *
     * Also, it simply doesn't work with the title ;). Not my falt btw.
     */

    fn get_string_property(&self, window: x::Window, property: x::Atom) -> Option<String> {
        let cookie = self.x_connection.send_request(&x::GetProperty {
            delete: false,
            window,
            property,
            r#type: x::ATOM_STRING,
            long_offset: 0,
            long_length: 0,
        });
        let reply = self.x_connection.wait_for_reply(cookie);
        let reply = if reply.is_err() {
            return None;
        } else {
            reply.unwrap()
        };
        let cookie = self.x_connection.send_request(&x::GetProperty {
            delete: false,
            window,
            property,
            r#type: x::ATOM_STRING,
            long_offset: 0,
            long_length: reply.bytes_after(),
        });
        let reply = self.x_connection.wait_for_reply(cookie);
        let reply = if reply.is_err() {
            return None;
        } else {
            reply.unwrap()
        };
        let mut replied_value = reply.value().to_vec();
        replied_value.pop();
        let prop = if let Ok(prop) = String::from_utf8(replied_value) {
            prop
        } else {
            return None;
        };
        Some(prop)
    }

    fn get_class(&self, window: x::Window) -> Option<(String, String)> {
        let (class1, class2) =
            if let Some(class) = self.get_string_property(window, x::ATOM_WM_CLASS) {
                let mut classes = class.split('\0');
                (
                    classes.next().unwrap().to_string(),
                    classes.next().unwrap().to_string(),
                )
            } else {
                return None;
            };

        Some((class1, class2))
    }
}
