use crate::*;
use xcb::x;
use xcb::Xid;

pub struct Screen {
    pub workspaces: Vec<Workspace>,
    pub current_wk: usize,
    pub root: x::Window,
    pub width: u16,
    pub height: u16,
}

impl Screen {
    pub fn new(
        lapin: &Lapin,
        root: x::Window,
        modmask: x::ModMask,
        keybinds: &KeybindSet,
        width: u16,
        height: u16,
    ) -> Self {
        lapin.x_connection.send_request(&x::GrabButton {
            owner_events: true,
            grab_window: root,
            event_mask: x::EventMask::BUTTON_PRESS | x::EventMask::BUTTON_RELEASE,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: x::Window::none(),
            cursor: x::Cursor::none(),
            button: x::ButtonIndex::Any,
            modifiers: modmask,
        });
        lapin.x_connection.send_request(&x::GrabButton {
            owner_events: true,
            grab_window: root,
            event_mask: x::EventMask::BUTTON_PRESS | x::EventMask::BUTTON_RELEASE,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: x::Window::none(),
            cursor: x::Cursor::none(),
            button: x::ButtonIndex::Any,
            modifiers: modmask | x::ModMask::SHIFT,
        });

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

        Screen {
            workspaces,
            root,
            current_wk: 0,
            width,
            height,
        }
    }

    /// Gets the current workspace struct of the screen.
    pub fn current_workspace(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current_wk]
    }
}

pub struct Workspace {
    pub name: &'static str,
    pub focused: Option<usize>,
    pub windows: Vec<x::Window>,
}

impl Workspace {
    pub fn new(name: &'static str) -> Self {
        Workspace {
            name,
            focused: None,
            windows: Vec::new(),
        }
    }
}
