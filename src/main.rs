extern crate dbus;
extern crate notify_rust as notify;

use dbus::arg;
use dbus::ConnectionItem;
use std::collections::BTreeMap;
use notify::{NotificationUrgency, NotificationHint};

static BT_ICON_ON: &'static str = "/usr/share/icons/oxygen/32x32/apps/preferences-system-bluetooth.png";
static BT_ICON_OFF: &'static str = "/usr/share/icons/oxygen/32x32/apps/preferences-system-bluetooth-inactive.png";


fn main() {
    let conn = dbus::Connection::get_private(dbus::BusType::System).unwrap();
    conn.add_match("type='signal',arg0='org.bluez.Device1',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path_namespace='/org/bluez/hci0'").unwrap();

    for ev in conn.iter(1000) {
        match ev {
            ConnectionItem::Signal(msg) => if let (Some(_), Some(props)) = msg.get2::<String, arg::Dict<String, arg::Variant<bool>, _>>() {
                let map = props.map(|(k, arg::Variant(v))| (k, v)).collect::<BTreeMap<String, bool>>();
                if let (Some(state), Some(path)) = (map.get("Connected"), msg.path()) {
                    let msg = dbus::Message::new_method_call("org.bluez", path, "org.freedesktop.DBus.Properties", "Get").unwrap().append2("org.bluez.Device1", "Name");
                    let name = match conn.send_with_reply_and_block(msg, 1000).ok().and_then(|r| r.get1::<arg::Variant<String>>()) {
                        Some(arg::Variant(name)) => name,
                        _ => String::from("Unknown"),
                    };

                    let _ = notify::Notification::new()
                        .summary("Bluetooth device")
                        .hint(NotificationHint::Urgency(NotificationUrgency::Low))
                        .icon(if *state {BT_ICON_ON} else {BT_ICON_OFF})
                        .body(&*format!("{} {}", name, if *state {"connected"} else {"disconnected"}))
                        .show();
                }
            },
            _ => continue
        }
    }
}
