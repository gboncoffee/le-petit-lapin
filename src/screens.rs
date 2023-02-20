use crate::*;
use xcb::x;

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

/// Will be a physical screen when multi-monitor support arrive.
pub struct Screen {
    pub workspaces: Vec<Workspace>,
    pub current_wk: usize,
    pub root: x::Window,
    pub width: u32,
    pub height: u32,
    pub atoms: Atoms,
}

impl Screen {
    /// Creates a screen. The `Lapin::init()` should call it for every monitor.
    /// Only use it manually if you know what you're doing.
    pub fn new(lapin: &Lapin, root: x::Window, keybinds: &KeybindSet) -> Self {
        // bind keys.
        for ((modmask, _, code), _) in keybinds.iter() {
            lapin.x_connection.send_request(&x::GrabKey {
                owner_events: true,
                grab_window: root,
                modifiers: *modmask,
                key: *code,
                pointer_mode: x::GrabMode::Async,
                keyboard_mode: x::GrabMode::Async,
            });
        }

        lapin.x_connection.send_request(&x::GrabButton {
            owner_events: true,
            grab_window: root,
            event_mask: x::EventMask::BUTTON_MOTION
                | x::EventMask::BUTTON_PRESS
                | x::EventMask::BUTTON_RELEASE,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: x::WINDOW_NONE,
            cursor: x::CURSOR_NONE,
            button: x::ButtonIndex::Any,
            modifiers: keys::match_mods(lapin.config.mouse_mod).0,
        });

        let event_mask = x::EventMask::SUBSTRUCTURE_NOTIFY
            | x::EventMask::STRUCTURE_NOTIFY
            | x::EventMask::SUBSTRUCTURE_REDIRECT
            | x::EventMask::PROPERTY_CHANGE;

        lapin.x_connection.send_request(&x::ChangeWindowAttributes {
            window: root,
            value_list: &[x::Cw::EventMask(event_mask)],
        });

        let mut workspaces = Vec::with_capacity(lapin.config.workspaces.len());
        for workspace in lapin.config.workspaces {
            workspaces.push(Workspace::new(workspace));
        }

        lapin.x_connection.flush().ok();

        // get width and height
        let cookie = lapin.x_connection.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(root),
        });
        let reply = lapin
            .x_connection
            .wait_for_reply(cookie)
            .expect("Failed to get screen geometry");
        let width = reply.width() as u32;
        let height = reply.height() as u32;

        let atoms = Atoms::intern_all(&lapin.x_connection).expect("Cannot init atoms!");

        Screen {
            workspaces,
            root,
            current_wk: 0,
            width,
            height,
            atoms,
        }
    }

    /// Gets the current workspace struct of the screen.
    pub fn current_workspace(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current_wk]
    }
}

/// A virtual workspace.
pub struct Workspace {
    pub name: &'static str,
    pub focused: Option<usize>,
    pub ool_focus: bool,
    pub windows: Vec<x::Window>,
    pub ool_windows: Vec<x::Window>,
    pub layout: usize,
}
impl Workspace {
    /// Creates a new workspace. The `Lapin::init()` function should create then
    /// based in the config struct. Only create then manually if you know what
    /// you're doing.
    pub fn new(name: &'static str) -> Self {
        Workspace {
            name,
            focused: None,
            ool_focus: false,
            windows: Vec::new(),
            ool_windows: Vec::new(),
            layout: 0,
        }
    }
}
