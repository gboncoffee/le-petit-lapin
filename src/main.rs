use lapin::keys::*;
use lapin::*;

fn main() {
    let mut lapin = Lapin::connect();

    const MODKEY: &str = "Meta";
    const TERMINAL: &str = "alacritty";

    let mut keybinds = KeybindSet::new();
    keybinds.bindall(vec![
        (&[MODKEY], "q", lazy! {Lapin::quit()}),
        (&[MODKEY], "Return", lazy! {Lapin::spawn(TERMINAL)}),
        (&[MODKEY], "n", lazy! {Lapin::spawn("chromium")}),
        (&[MODKEY], "a", lazy! {Lapin::spawn("rofi -show run")}),
        (&[MODKEY], "w", lazy! {wm, wm.killfocused()}),
    ]);

    lapin.config.mouse_mod = &[MODKEY];

    lapin.init(&mut keybinds);
}
