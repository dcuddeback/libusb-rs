use libusb::*;

#[derive(Debug, Clone, Copy)]
pub enum HotPlugEvent {
    Arrived,
    Left,
}

impl HotPlugEvent {
    pub fn from_i32(value: i32) -> Option<HotPlugEvent> {
        match value {
            e if e == LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED => Some(HotPlugEvent::Arrived),
            e if e == LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT =>  Some(HotPlugEvent::Left),
            _ => None,
        }
    }
}
