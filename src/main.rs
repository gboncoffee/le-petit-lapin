use lapin::keys::*;
use lapin::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let lapin = Rc::new(RefCell::new(Lapin::connect()));

    const MODKEY: &str = "Meta";
    const TERMINAL: &str = "alacritty";

    let keys = vec![
        key!([MODKEY], "q", || quit()),
        key!([MODKEY], "Return", || spawn(TERMINAL)),
        key!([MODKEY], "n", || spawn("chromium")),
        key!([MODKEY], "a", || spawn("rofi -show run")),
        key!([MODKEY], "w", || lapin.borrow().killfocused()),
    ];

    lapin.borrow_mut().init();
}
