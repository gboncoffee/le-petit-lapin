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
    pub screens: Vec<Screen>,
    current_scr: usize,
    atoms: Option<Atoms>,
}

impl fmt::Display for Lapin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_screens = self.screens.len();
        let cur_screen = self.current_scr;
        let total_workspaces = self.screens[cur_screen].workspaces.len();
        let cur_workspace = self.screens[cur_screen].current_wk;
        let total_windows = self.screens[cur_screen].workspaces[cur_workspace]
            .windows
            .len();
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

    fn window_location(&self, win: x::Window) -> Option<(usize, usize, usize)> {
        for (s, screen) in self.screens.iter().enumerate() {
            for (k, workspace) in screen.workspaces.iter().enumerate() {
                for (w, window) in workspace.windows.iter().enumerate() {
                    if *window == win {
                        return Some((s, k, w));
                    }
                }
            }
        }
        None
    }

    fn manage_window(&mut self, ev: x::MapRequestEvent) {
        self.x_connection.send_request(&x::ChangeWindowAttributes {
            window: ev.window(),
            value_list: &[
                x::Cw::BorderPixel(self.config.border_color),
                x::Cw::EventMask(
                    x::EventMask::ENTER_WINDOW
                        | x::EventMask::LEAVE_WINDOW
                        | x::EventMask::PROPERTY_CHANGE
                        | x::EventMask::STRUCTURE_NOTIFY,
                ),
            ],
        });

        if self.current_layout().draw_borders() {
            self.x_connection.send_request(&x::ConfigureWindow {
                window: ev.window(),
                value_list: &[x::ConfigWindow::BorderWidth(self.config.border_width)],
            });
        }

        let scr = self.current_scr;
        let wk = self.screens[scr].current_wk;
        self.screens[scr].workspaces[wk]
            .windows
            .insert(0, ev.window());

        self.current_layout()
            .newwin(&mut self.workspace_windows(), &self.x_connection);

        self.x_connection.send_request(&x::MapWindow {
            window: ev.window(),
        });

        self.x_connection.flush().ok();
    }

    fn unmanage_window(&mut self, window: x::Window) {
        if let Some((s, k, w)) = self.window_location(window) {
            self.screens[s].workspaces[k].windows.remove(w);
            self.screens[s].workspaces[k].focused = None;
            let n_wins = self.screens[s].workspaces[k].windows.len();
            if n_wins > 0 {
                let win = if w != 0 {
                    if w == n_wins {
                        w - 1
                    } else {
                        w
                    }
                } else {
                    0
                };
                self.set_focus(self.screens[s].workspaces[k].windows[win], s, k, win);
            }
            self.current_layout().delwin(
                &mut self.workspace_windows(),
                self.current_workspace().focused,
                &self.x_connection,
            );
        }
    }

    fn set_focus(&mut self, window: x::Window, s: usize, k: usize, w: usize) {
        self.current_scr = s;
        self.screens[s].current_wk = k;
        self.screens[s].workspaces[k].focused = Some(w);
        self.x_connection.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        });
        self.x_connection.send_request(&x::ConfigureWindow {
            window,
            value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
        });
        if self.current_layout().draw_borders() {
            self.x_connection.send_request(&x::ChangeWindowAttributes {
                window,
                value_list: &[x::Cw::BorderPixel(self.config.border_color_focus)],
            });
        }
        self.x_connection.flush().ok();
    }

    fn toggle_focus(&mut self, window: x::Window) {
        if let Some((s, k, w)) = self.window_location(window) {
            self.set_focus(window, s, k, w);
        }
    }

    fn restore_border(&self, window: x::Window) {
        if self.current_layout().draw_borders() {
            self.x_connection.send_request(&x::ChangeWindowAttributes {
                window,
                value_list: &[x::Cw::BorderPixel(self.config.border_color)],
            });
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
        let k = self.screens[s].current_wk;
        let n_wins = self.screens[s].workspaces[k].windows.len();
        if n_wins > 1 {
            if let Some(win) = self.get_focused_window() {
                self.restore_border(win);
            }
            self.screens[s].workspaces[k].focused =
                if let Some(cwin) = self.screens[s].workspaces[k].focused {
                    let new_n = if previous && cwin > 0 {
                        cwin - 1
                    } else if previous {
                        n_wins - 1
                    } else {
                        cwin + 1
                    };
                    if new_n >= n_wins {
                        Some(0)
                    } else {
                        Some(new_n)
                    }
                } else {
                    Some(0)
                };
            let w_n = self.screens[s].workspaces[k].focused.unwrap();
            let window = self.screens[s].workspaces[k].windows[w_n];
            self.set_focus(window, s, k, w_n);
            self.current_layout().changewin(
                &mut self.workspace_windows(),
                w_n,
                &self.x_connection,
                previous,
            );
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

        loop {
            match utils::get_x_event(&self.x_connection) {
                x::Event::MapRequest(ev) => self.manage_window(ev),
                x::Event::DestroyNotify(ev) => self.unmanage_window(ev.window()),
                x::Event::EnterNotify(ev) => self.toggle_focus(ev.event()),
                x::Event::LeaveNotify(ev) => self.restore_border(ev.event()),
                x::Event::KeyPress(ev) => {
                    if let Some(callback) = keybinds.get_callback(ev.detail(), ev.state()) {
                        callback(self);
                    }
                }
                x::Event::ButtonPress(ev) => {
                    if self.current_layout().allow_motions() {
                        (diff_x, diff_y, pos_x, pos_y, move_window) = self.init_mouse_action(&ev)
                    }
                }
                x::Event::ButtonRelease(_) => {
                    if self.current_layout().allow_motions() {
                        (diff_x, diff_y) = (None, None)
                    }
                }
                x::Event::MotionNotify(ev) => {
                    if self.current_layout().allow_motions() {
                        if let Some(x_d) = diff_x {
                            let y_d = diff_y.unwrap();
                            let x_p = pos_x.unwrap();
                            let y_p = pos_y.unwrap();
                            let win = move_window.unwrap();
                            self.handle_motion(ev, x_d, y_d, x_p, y_p, win);
                        }
                    }
                }
                // other => println!("{:?}", other),
                _ => {}
            }
        }
    }

    fn current_screen<'a>(&'a self) -> &'a Screen {
        &self.screens[self.current_scr]
    }

    fn current_workspace<'a>(&'a self) -> &'a Workspace {
        let wk = self.current_screen().current_wk;
        &self.current_screen().workspaces[wk]
    }

    fn current_layout<'a>(&'a self) -> &'a Box<dyn layouts::Layout> {
        let layout = self.current_workspace().layout;
        &self.config.layouts[layout]
    }

    fn workspace_windows<'a>(&'a self) -> std::slice::Iter<'a, x::Window> {
        self.current_workspace().windows.iter()
    }
}
