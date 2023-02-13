use xcb;

pub fn get_x_event(con: &xcb::Connection) -> xcb::x::Event {
    loop {
        let event = con
            .wait_for_event()
            .expect("Connection to the X server failed!");
        if let xcb::Event::X(ev) = event {
            return ev;
        }
    }
}
