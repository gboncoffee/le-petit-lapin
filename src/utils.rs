use xcb;

pub fn get_x_event(con: &xcb::Connection) -> xcb::x::Event {
    loop {
        if let Ok(event) = con.wait_for_event() {
            if let xcb::Event::X(ev) = event {
                return ev;
            }
        }
    }
}
