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
//!     // You can autostart stuff easily using the Lapin::spawn.
//!     Lapin::spawn("picom");
//!
//!     // The last thing to do is starting the window manager with the keybind
//!     // set with Lapin::init(&mut KeybindSet).
//!     lapin.init(&mut keybinds);
//! }
//! ```

pub mod config;
pub mod keys;
pub mod lapin_api;
pub mod layouts;
pub mod screens;
pub mod utils;

use config::*;
use keys::*;
use screens::*;
use std::fmt;
use std::time;
use xcb::x;
use xcb::Connection;
use xcb::Xid;

xcb::atoms_struct! {
    #[derive(Copy, Clone, Debug)]
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
    }
}

/// The window manager I suppose.
pub struct Lapin {
    pub x_connection: Connection,
    pub config: Config,
    pub keybinds: KeybindSet,
    pub screens: Vec<Screen>,
    pub atoms: Atoms,
    current_scr: usize,
    root: x::Window,
}

impl fmt::Display for Lapin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_screens = self.screens.len();
        let cur_screen = self.current_scr;
        let total_workspaces = self.current_screen().workspaces.len();
        let cur_workspace = self.current_screen().current_wk;
        let total_windows = self.current_workspace().windows.len();
        // gambiarra
        let cur_window =
            if let Some(win) = self.screens[cur_screen].workspaces[cur_workspace].focused {
                format!("{win}")
            } else {
                "N/A".to_string()
            };

        f.write_str(&format!("Screen: {cur_screen}/{total_screens}, Workspace: {cur_workspace}/{total_workspaces}, Window: {cur_window}/{total_windows}"))
    }
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
            r#type: self.atoms.net_client_list,
            data: &window.resource_id().to_ne_bytes(),
        });
    }

    fn manage_window(&mut self, ev: x::MapRequestEvent) {
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

        self.add_border(ev.window());
        if let Some(old_win) = self.get_focused_window() {
            self.restore_border(old_win);
        }

        self.current_workspace_mut().windows.insert(0, ev.window());

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

        self.add_client_to_atom(ev.window());

        self.x_connection.flush().ok();

        self.set_focus(
            ev.window(),
            self.current_scr,
            self.current_screen().current_wk,
            0,
            false,
        );

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
        self.set_focus(window, s, k, w, ool);
    }

    fn unmanage_window(&mut self, window: x::Window) {
        if let Some((s, k, w, ool)) = self.window_location(window) {
            if ool {
                self.current_workspace_mut().ool_windows.remove(w);
            } else {
                self.current_workspace_mut().windows.remove(w);
            }
            self.current_workspace_mut().focused = None;
            self.current_layout().delwin(
                &mut self.workspace_windows(),
                self.current_workspace().focused,
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y,
            );
            self.x_connection.send_request(&x::ChangeProperty::<u8> {
                mode: x::PropMode::Replace,
                window: self.root,
                property: self.atoms.net_client_list,
                r#type: self.atoms.net_client_list,
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
            self.reset_focus_after_removing(s, k, w, ool);
            self.current_layout().delwin(
                &mut self.workspace_windows(),
                self.current_workspace().focused,
                &self.x_connection,
                self.current_screen().width,
                self.current_screen().height,
                self.current_screen().x,
                self.current_screen().y
            );
            self.x_connection.flush().ok();
        }
    }

    fn set_focus(&mut self, window: x::Window, s: usize, k: usize, w: usize, ool: bool) {
        self.current_scr = s;
        self.current_screen_mut().current_wk = k;
        self.current_workspace_mut().focused = Some(w);
        self.current_workspace_mut().ool_focus = ool;
        self.x_connection.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        });
        self.x_connection.send_request(&x::ConfigureWindow {
            window,
            value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
        });
        self.color_focused_border(window);
        self.x_connection.flush().ok();
    }

    fn toggle_focus(&mut self, window: x::Window) {
        if let Some((s, k, w, ool)) = self.window_location(window) {
            self.set_focus(window, s, k, w, ool);
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
            self.set_focus(window, s, k, new_w, ool);
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
        if let Some(window) = self.get_focused_window() {
            self.color_focused_border(window);
            if !self.current_workspace().ool_focus {
                let n = self.current_workspace().focused.unwrap();
                self.current_layout().changewin(
                    &mut self.workspace_windows(),
                    n,
                    &self.x_connection,
                    self.current_screen().width,
                    self.current_screen().height,
                    self.current_screen().x,
                    self.current_screen().y,
                )
            }
        }
        self.x_connection.flush().ok();
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

        loop {
            match utils::get_x_event(&self.x_connection) {
                x::Event::MapRequest(ev) => {
                    last_map = time::SystemTime::now();
                    self.manage_window(ev);
                }
                x::Event::DestroyNotify(ev) => {
                    last_map = time::SystemTime::now();
                    self.unmanage_window(ev.window());
                }
                x::Event::EnterNotify(ev) => {
                    if time::SystemTime::now().duration_since(last_map).unwrap()
                        > time::Duration::from_millis(100)
                    {
                        if let Some(old_win) = self.get_focused_window() {
                            self.restore_border(old_win);
                        }
                        self.toggle_focus(ev.event());
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
}
