use lapin::keys::*;
use lapin::layouts::*;
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
        (&[MODKEY], "j", lazy! {wm, wm.nextwin()}),
        (&[MODKEY], "k", lazy! {wm, wm.prevwin()}),
        (&[MODKEY], "space", lazy! {wm, wm.next_layout()}),
        (&[MODKEY, "Shift"], "space", lazy! {wm, wm.prev_layout()}),
    ]);

    lapin.config.mouse_mod = &[MODKEY];

    let tile = Tiling {
        name: "tile",
        borders: 4,
        master_factor: 1.0 / 2.0,
        gaps: 4,
    };
    let max = Maximized {
        name: "max",
        borders: 4,
        gaps: 4,
    };
    let float = Floating {
        name: "float",
        borders: 4,
    };

    lapin.config.layouts = layouts![tile, max, float];

    // Lapin::spawn("picom");

    lapin.init(&mut keybinds);
}
