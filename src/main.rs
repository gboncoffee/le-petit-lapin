use lapin::keys::*;
use lapin::layouts::*;
use lapin::*;

fn main() {
    let mut lapin = Lapin::connect();

    const MODKEY: &str = "Meta";
    const TERMINAL: &str = "alacritty";

    let mut keybinds = KeybindSet::new();
    keybinds.bindall(vec![
        (&[MODKEY], "1", lazy! {wm, wm.goto_workspace(1)}),
        (&[MODKEY], "2", lazy! {wm, wm.goto_workspace(2)}),
        (&[MODKEY], "3", lazy! {wm, wm.goto_workspace(3)}),
        (&[MODKEY], "4", lazy! {wm, wm.goto_workspace(4)}),
        (&[MODKEY], "5", lazy! {wm, wm.goto_workspace(5)}),
        (&[MODKEY], "6", lazy! {wm, wm.goto_workspace(6)}),
        (&[MODKEY], "7", lazy! {wm, wm.goto_workspace(7)}),
        (&[MODKEY], "8", lazy! {wm, wm.goto_workspace(8)}),
        (&[MODKEY], "9", lazy! {wm, wm.goto_workspace(9)}),
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
        gaps_on_single: false,
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
