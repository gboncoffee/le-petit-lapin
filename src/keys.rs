use crate::*;
use std::collections::hash_map;
use std::collections::HashMap;
use x11::xlib;
use x11rb::protocol::xproto as xp;

pub type Callback = Box<dyn FnMut(&mut Lapin) -> ()>;

pub fn match_butmask_with_modmask(modkey: xp::KeyButMask) -> xp::ModMask {
    match modkey {
        xp::KeyButMask::SHIFT => xp::ModMask::SHIFT,
        xp::KeyButMask::CONTROL => xp::ModMask::CONTROL,
        xp::KeyButMask::LOCK => xp::ModMask::LOCK,
        xp::KeyButMask::MOD1 => xp::ModMask::M1,
        xp::KeyButMask::MOD2 => xp::ModMask::M2,
        xp::KeyButMask::MOD4 => xp::ModMask::M4,
        _ => panic!("Please never create two types to represent the same fucking thing"),
    }
}

/// Matches a modkey name with it's mod mask value.
///
/// # Panics:
/// This function panics if there's no such modkey.
pub fn match_mod(modkey: &str) -> (xp::ModMask, xp::KeyButMask) {
    match &modkey.to_uppercase()[..] {
        "META" | "ALT" => (
            match_butmask_with_modmask(xp::KeyButMask::MOD1),
            xp::KeyButMask::MOD1,
        ),
        "SUPER" | "WIN" | "HYPER" => (
            match_butmask_with_modmask(xp::KeyButMask::MOD4),
            xp::KeyButMask::MOD4,
        ),
        "LOCK" => (
            match_butmask_with_modmask(xp::KeyButMask::LOCK),
            xp::KeyButMask::LOCK,
        ),
        "CTRL" | "CONTROL" => (
            match_butmask_with_modmask(xp::KeyButMask::CONTROL),
            xp::KeyButMask::CONTROL,
        ),
        other => panic!("No such modkey {other} or modkey not allowed"),
    }
}

/// Same but with a list.
pub fn match_mods(mods: &[&str]) -> (xp::ModMask, xp::KeyButMask) {
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

/// Keybind set.
pub struct KeybindSet {
    map: HashMap<(xp::ModMask, xp::KeyButMask, xp::Keycode), Callback>,
}

impl KeybindSet {
    /// Creates a new empty keybind set.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

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

    pub fn get_callback(
        &mut self,
        code: xp::Keycode,
        modmask: xp::KeyButMask,
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

    pub fn iter(
        &self,
    ) -> hash_map::Iter<(xp::ModMask, xp::KeyButMask, u8), Box<dyn FnMut(&mut Lapin)>> {
        self.map.iter()
    }
}

#[macro_export]
macro_rules! lazy {
    ($callback:expr) => {
        Box::new(|_: &mut Lapin| $callback) as Callback
    };
    ($name:ident, $callback:expr) => {
        Box::new(|$name: &mut Lapin| $callback) as Callback
    };
}
