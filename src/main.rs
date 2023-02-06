use lapin::*;

fn main() {
    let lapin = Lapin::connect();

    const MODKEY: &str = "Meta";
    const TERMINAL: &str = "alacritty";

    const KEYS: Keylist = &[
        key!(&[MODKEY], "q", &|| quit()),
        key!(&[MODKEY], "Return", &|| spawn(TERMINAL)),
        key!(&[MODKEY], "n", &|| spawn("chromium")),
        key!(&[MODKEY], "a", &|| spawn("rofi -show run")),
    ];

    init(lapin, KEYS);
}
