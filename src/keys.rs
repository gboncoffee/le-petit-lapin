//! Keybind system

use crate::*;
use std::collections::hash_map;
use std::collections::HashMap;
use x11::xlib;
use xcb::x;

/// A closure callable by a keybind.
pub type Callback = Box<dyn FnMut(&mut Lapin) -> ()>;

/// Matches a `xcb::x::KeyButMask` with a `xcb::x::ModMask`.
fn match_butmask_with_modmask(modkey: x::KeyButMask) -> x::ModMask {
    let mut modmask = x::ModMask::empty();
    if modkey.contains(x::KeyButMask::SHIFT) {
        modmask = modmask | x::ModMask::SHIFT;
    }
    if modkey.contains(x::KeyButMask::CONTROL) {
        modmask = modmask | x::ModMask::CONTROL;
    }
    if modkey.contains(x::KeyButMask::LOCK) {
        modmask = modmask | x::ModMask::LOCK;
    }
    if modkey.contains(x::KeyButMask::MOD1) {
        modmask = modmask | x::ModMask::N1;
    }
    if modkey.contains(x::KeyButMask::MOD2) {
        modmask = modmask | x::ModMask::N2;
    }
    if modkey.contains(x::KeyButMask::MOD4) {
        modmask = modmask | x::ModMask::N4;
    }
    modmask
}

/// Matches a modkey name with it's mod mask value.
///
/// # Panics:
/// This function panics if there's no such modkey.
fn match_mod(modkey: &str) -> (x::ModMask, x::KeyButMask) {
    match &modkey.to_uppercase()[..] {
        "META" | "ALT" => (
            match_butmask_with_modmask(x::KeyButMask::MOD1),
            x::KeyButMask::MOD1,
        ),
        "SUPER" | "WIN" | "HYPER" => (
            match_butmask_with_modmask(x::KeyButMask::MOD4),
            x::KeyButMask::MOD4,
        ),
        "LOCK" => (
            match_butmask_with_modmask(x::KeyButMask::LOCK),
            x::KeyButMask::LOCK,
        ),
        "CTRL" | "CONTROL" => (
            match_butmask_with_modmask(x::KeyButMask::CONTROL),
            x::KeyButMask::CONTROL,
        ),
        "SHIFT" => (
            match_butmask_with_modmask(x::KeyButMask::SHIFT),
            x::KeyButMask::SHIFT,
        ),
        other => panic!("No such modkey {other} or modkey not allowed"),
    }
}

/// Matches a list of modifier key names with modifier masks from `xcb::x`.
///
/// # Panics
///
/// This function panics if it encounters a invalid modkey.
pub fn match_mods(mods: &[&str]) -> (x::ModMask, x::KeyButMask) {
    let mut moditer = mods.iter();
    let mut modmask = match_mod(moditer.next().expect("At least one modkey is required")).0;
    for newmod in moditer {
        modmask = modmask | match_mod(newmod).0;
    }
    let mut moditer = mods.iter();
    let mut butmodmask = match_mod(moditer.next().unwrap()).1;
    for newmod in moditer {
        butmodmask = butmodmask | match_mod(newmod).1;
    }
    (modmask, butmodmask)
}

/// The keybind set.
pub struct KeybindSet {
    map: HashMap<(x::ModMask, x::KeyButMask, x::Keycode), Callback>,
}

impl KeybindSet {
    /// Creates a new empty keybind set.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Binds all keybinds in a vector.
    ///
    /// # Example
    /// ```no_run
    /// use le_petit_lapin::keys::*;
    /// use le_petit_lapin::*;
    /// let mut keybinds = KeybindSet::new();
    /// keybinds.bindall(vec![
    ///     (&["Super"], "1", lazy! {wm, wm.goto_workspace(1)}),
    ///     (&["Super"], "Return", lazy! {Lapin::spawn("alacritty")}),
    ///     (&["Super", "Shift"], "Return", lazy! {wm, wm.change_master()}),
    /// ]);
    ///```
    pub fn bindall(&mut self, keys: Vec<(&[&str], &str, Callback)>) {
        // I'm extremelly angry that I must use unsafe to call C code to do
        // this basic stuff. Rust port of X libraries is still shit. I'm so
        // mad like holy fucking shit.
        let xlib_display = unsafe { xlib::XOpenDisplay(std::ptr::null_mut()) };
        for (mods, key, callback) in keys {
            let keycode = unsafe {
                let cstr = std::ffi::CString::new(key).unwrap();
                let tmp_ptr: Vec<u8> = cstr.into_bytes_with_nul();
                let mut ptr: Vec<i8> = tmp_ptr.into_iter().map(|c| c as i8).collect();
                xlib::XKeysymToKeycode(xlib_display, xlib::XStringToKeysym(ptr.as_mut_ptr()))
            };
            let (modmask, keybutmask) = match_mods(mods);
            self.map.insert((modmask, keybutmask, keycode), callback);
        }
    }

    /// Returns the closure from a keybind.
    pub fn get_callback(
        &mut self,
        code: x::Keycode,
        modmask: x::KeyButMask,
    ) -> Option<&mut Callback> {
        if let Some(callback) =
            self.map
                .get_mut(&(match_butmask_with_modmask(modmask), modmask, code))
        {
            Some(callback)
        } else {
            None
        }
    }

    /// Returns an iterator on the keybinds.
    pub fn iter(&self) -> hash_map::Iter<(x::ModMask, x::KeyButMask, u8), Callback> {
        self.map.iter()
    }
}

/// Creates a closure suitable to use in keybinds.
///
/// # Example
/// ```no_run
/// use le_petit_lapin::keys::*;
/// use le_petit_lapin::*;
/// let mut keybinds = KeybindSet::new();
/// keybinds.bindall(vec![
///     // closure that calls the main `Lapin` struct.
///     (&["Super"], "1", lazy! {wm, wm.goto_workspace(1)}),
///     // closure that does not call it.
///     (&["Super"], "Return", lazy! {Lapin::spawn("alacritty")}),
///     // closures that do a lot of stuff.
///     (&["Super", "Shift"], "Return", lazy! {{
///         Lapin::spawn("alacritty");
///         Lapin::spawn("notify-send welcome back to the terminal!");
///     }}),
///     (&["Super", "Meta"], "space", lazy! {wm, {
///         wm.goto_workspace(5);
///         Lapin::spawn("chromium");
///     }}),
/// ]);
/// ```
#[macro_export]
macro_rules! lazy {
    ($callback:expr) => {
        Box::new(|_: &mut Lapin| $callback) as Callback
    };
    ($name:ident, $callback:expr) => {
        Box::new(|$name: &mut Lapin| $callback) as Callback
    };
}
