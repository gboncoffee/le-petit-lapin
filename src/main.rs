use lapin::keys::*;
use lapin::layouts::*;
use lapin::*;

fn main() {
    let mut lapin = Lapin::connect();

    const MODKEY: &str = "Meta";
    const TERMINAL: &str = "alacritty";

    let mut keybinds = KeybindSet::new();
    keybinds.bindall(vec![
        // workspace keys
        (&[MODKEY], "1", lazy! {wm, wm.goto_workspace(0)}),
        (&[MODKEY], "2", lazy! {wm, wm.goto_workspace(1)}),
        (&[MODKEY], "3", lazy! {wm, wm.goto_workspace(2)}),
        (&[MODKEY], "4", lazy! {wm, wm.goto_workspace(3)}),
        (&[MODKEY], "5", lazy! {wm, wm.goto_workspace(4)}),
        (&[MODKEY], "6", lazy! {wm, wm.goto_workspace(5)}),
        (&[MODKEY], "7", lazy! {wm, wm.goto_workspace(6)}),
        (&[MODKEY], "8", lazy! {wm, wm.goto_workspace(7)}),
        (&[MODKEY], "9", lazy! {wm, wm.goto_workspace(8)}),
        (&[MODKEY, "Shift"], "1", lazy! {wm, wm.send_window_to_workspace(0)}),
        (&[MODKEY, "Shift"], "2", lazy! {wm, wm.send_window_to_workspace(1)}),
        (&[MODKEY, "Shift"], "3", lazy! {wm, wm.send_window_to_workspace(2)}),
        (&[MODKEY, "Shift"], "4", lazy! {wm, wm.send_window_to_workspace(3)}),
        (&[MODKEY, "Shift"], "5", lazy! {wm, wm.send_window_to_workspace(4)}),
        (&[MODKEY, "Shift"], "6", lazy! {wm, wm.send_window_to_workspace(5)}),
        (&[MODKEY, "Shift"], "7", lazy! {wm, wm.send_window_to_workspace(6)}),
        (&[MODKEY, "Shift"], "8", lazy! {wm, wm.send_window_to_workspace(7)}),
        (&[MODKEY, "Shift"], "9", lazy! {wm, wm.send_window_to_workspace(8)}),
        // quit
        (&[MODKEY], "q", lazy! {Lapin::quit()}),
        // spawns
        (&[MODKEY], "Return", lazy! {Lapin::spawn(TERMINAL)}),
        (&[MODKEY], "n", lazy! {Lapin::spawn("chromium")}),
        (&[MODKEY], "a", lazy! {Lapin::spawn("rofi -show run")}),
        // kill focus
        (&[MODKEY], "w", lazy! {wm, wm.killfocused()}),
        // change focus
        (&[MODKEY], "j", lazy! {wm, wm.nextwin()}),
        (&[MODKEY], "k", lazy! {wm, wm.prevwin()}),
        // change layout
        (&[MODKEY], "space", lazy! {wm, wm.next_layout()}),
        (&[MODKEY, "Shift"], "space", lazy! {wm, wm.prev_layout()}),
        // swap slaves
        (
            &[MODKEY, "Shift"],
            "k",
            lazy! {wm, wm.swap_with_prev_slave()},
        ),
        (
            &[MODKEY, "Shift"],
            "j",
            lazy! {wm, wm.swap_with_next_slave()},
        ),
        // change master
        (&[MODKEY, "Shift"], "Return", lazy! {wm, wm.change_master()}),
        // toggle ool
        (&[MODKEY, "Shift"], "t", lazy! {wm, wm.toggle_ool()}),
        // fullscreen
        (&[MODKEY, "Shift"], "f", lazy! {wm, wm.fullscreen()}),
        // change focused screen (monitor)
        (&[MODKEY], "y", lazy! {wm, wm.prev_screen()}),
        (&[MODKEY], "u", lazy! {wm, wm.next_screen()}),
        // change focused window screen
        (&[MODKEY, "Shift"], "y", lazy! {wm, wm.send_window_to_prev_screen()}),
        (&[MODKEY, "Shift"], "u", lazy! {wm, wm.send_window_to_next_screen()}),
    ]);

    lapin.config.mouse_mod = &[MODKEY];

    let tile = Tiling::new();
    let max = Maximized::new();
    let float = Floating::new();

    lapin.config.layouts = layouts![tile, max, float];

    // Lapin::spawn("picom");

    lapin.init(&mut keybinds);
}
