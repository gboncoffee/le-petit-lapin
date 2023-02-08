use std::collections::HashMap;
use x11::xlib;
use x11rb::protocol::xproto as xp;
use xkbcommon::xkb;

/// Matches a modkey name with it's mod mask value.
///
/// # Panics:
/// This function panics if there's no such modkey.
pub fn match_mod(modkey: &str) -> (xp::ModMask, xp::KeyButMask) {
    match &modkey.to_uppercase()[..] {
        "META" | "ALT" => (xp::ModMask::M4, xp::KeyButMask::MOD1),
        "SUPER" | "WIN" | "HYPER" => (xp::ModMask::M4, xp::KeyButMask::MOD4),
        "LOCK" => (xp::ModMask::LOCK, xp::KeyButMask::LOCK),
        "CTRL" | "CONTROL" => (xp::ModMask::CONTROL, xp::KeyButMask::CONTROL),
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
    map: HashMap<(xp::ModMask, xp::KeyButMask, xp::Keycode), Box<dyn Fn() -> ()>>,
    keymap: xkb::Keymap,
}

impl KeybindSet {
    /// Creates a new empty keybind set.
    pub fn new() -> Self {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(&context, "", "", "", "", None, 0).unwrap();
        Self {
            map: HashMap::new(),
            keymap,
        }
    }

    pub fn bindall(keys: Vec<(&[&str], &str, Box<dyn Fn() -> ()>)>) {
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
        }
    }
}

#[macro_export]
macro_rules! key {
    ($mods:expr, $key:expr, $callback:expr) => {
        ($mods, $key, Box::new($callback) as Box<dyn Fn() -> ()>)
    };
}
