use crate::*;
use x11rb::protocol::xproto as xp;

pub struct Screen {
    pub workspaces: Vec<Workspace>,
    pub current_wk: usize,
    pub root: xp::Window,
}

impl Screen {
    pub fn new(lapin: &Lapin, root: xp::Window, modmask: xp::ModMask) -> Self {
        xp::grab_button(
            &lapin.x_connection,
            true,
            root,
            xp::EventMask::BUTTON_PRESS | xp::EventMask::BUTTON_RELEASE,
            xp::GrabMode::ASYNC,
            xp::GrabMode::ASYNC,
            root,
            root,
            xp::ButtonIndex::ANY,
            modmask | xp::ModMask::SHIFT,
        )
        .expect("Cannot grab the mouse!");

        let event_mask = xp::EventMask::SUBSTRUCTURE_NOTIFY
            | xp::EventMask::STRUCTURE_NOTIFY
            | xp::EventMask::SUBSTRUCTURE_REDIRECT
            | xp::EventMask::PROPERTY_CHANGE;
        xp::change_window_attributes(
            &lapin.x_connection,
            root,
            &xp::ChangeWindowAttributesAux::new().event_mask(event_mask),
        )
        .expect("Cannot change window attributes!");

        let mut workspaces = Vec::with_capacity(lapin.config.workspaces.len());
        for workspace in lapin.config.workspaces {
            workspaces.push(Workspace::new(workspace));
        }

        lapin
            .x_connection
            .flush()
            .expect("Connection to X server failed!");

        Screen {
            workspaces,
            root,
            current_wk: 0,
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
    pub windows: Vec<xp::Window>,
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
