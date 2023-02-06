pub mod config;

use config::*;
use keyfn;
use std::process;
use std::thread;
use x11rb::connection::Connection;
use x11rb::protocol::Event as XEvent;
use x11rb::*;
use xkbcommon::xkb;

/// Matches a modkey name with it's mod mask value.
///
/// # Panics:
/// This function panics if there's no such modkey.
fn match_mod(modkey: &str) -> keyfn::Mod {
    match &modkey.to_uppercase()[..] {
        "SUPER" | "WIN" => keyfn::Mod::Windows,
        "ALT" | "META" => keyfn::Mod::Alt,
        "CTRL" | "CONTROL" => keyfn::Mod::Control,
        "SHIFT" => keyfn::Mod::Shift,
        "NUMLOCK" => keyfn::Mod::NumLock,
        "CAPSLOCK" => keyfn::Mod::CapsLock,
        "SCROLLLOCK" => keyfn::Mod::ScrollLock,
        "MOD5" => keyfn::Mod::Mod5,
        _ => panic!("No such modifier: {modkey}"),
    }
}

/// The window manager I suppose.
pub struct Lapin {
    pub x_connection: rust_connection::RustConnection,
    pub config: Config,
}

impl Lapin {
    /// The first function that should be called: to connect the window manager
    /// to the X server.
    pub fn connect() -> Self {
        let (x_connection, _) = connect(None).expect("Cannot connect to the X server!");
        let config = Config::new();
        Lapin {
            x_connection,
            config,
        }
    }

    /// The main event loop of the window manager. Ignores every key press
    /// event as those are handled by keyfn.
    pub fn event_loop(&self) -> ! {
        loop {
            let event = self
                .x_connection
                .wait_for_event()
                .expect("Connection to X server was closed!");
            if let XEvent::KeyPress(_) = event {
                println!("Ignoring keypress");
                continue;
            }
            println!("Event received: {:?}", event);
        }
    }
}

/// Function to spawn a command.
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

/// Function to terminate the window manager process.
pub fn quit() {
    process::exit(0);
}

/// Keybind struct that can be binded on `init()`.
pub struct Key {
    mods: &'static [&'static str],
    key: &'static str,
    callback: &'static dyn Fn() -> (),
}

/// Keylist that is sent to `init()` to bind keys.
pub type Keylist = &'static [&'static Key];

impl Key {
    /// Creates a new Keybind.
    pub const fn new(
        mods: &'static [&'static str],
        key: &'static str,
        callback: &'static dyn Fn() -> (),
    ) -> Self {
        Key {
            mods,
            key,
            callback,
        }
    }
}

/// Returns a pointer to a `Key`. Basically a sugar to `&Key::new()`.
#[macro_export]
macro_rules! key {
    ($mods:expr, $key:expr, $callback:expr) => {
        &Key::new($mods, $key, $callback)
    };
}

/// Binds keys and inits the window manager, the last function that should be
/// called.
pub fn init(lapin: Lapin, keys: &'static [&'static Key]) {
    thread::spawn(move || lapin.event_loop());
    let mut key_storage = keyfn::KeyStorage::new();
    for key in keys {
        let keysym = xkb::keysym_from_name(key.key, xkb::KEYSYM_CASE_INSENSITIVE);
        if keysym == xkb::KEY_NoSymbol {
            panic!("No such key: {}", key.key);
        }
        let keybind = keyfn::KeyBind::new(
            keysym,
            key.mods.iter().map(|s| match_mod(s)).collect(),
            keyfn::Trigger::Pressed,
            key.callback,
        );
        key_storage.add(keybind);
    }
    key_storage.start();
}

#[cfg(test)]
mod tests {
    use super::*;
}
