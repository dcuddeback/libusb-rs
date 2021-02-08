use device::Device;
use event::HotPlugEvent;

use libusb::*;

pub struct CallbackWrapper {
    pub closure: Box<dyn Fn(&Device, HotPlugEvent)>,
    pub handle: i32,
}

#[derive(Default)]
pub struct HotplugFilter {
    vendor: Option<i32>,
    product: Option<i32>,
    class: Option<i32>,
    events: Option<i32>,
    enumerate: bool,
}

impl HotplugFilter {
    pub fn new() -> Self {
        Self {
            vendor: None,
            product: None,
            class: None,
            events: None,
            enumerate: false,
        }
    }

    pub(crate) fn get_vendor(&self) -> i32 {
        self.vendor.unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY)
    }

    pub(crate) fn get_product(&self) -> i32 {
        self.product.unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY)
    }

    pub(crate) fn get_class(&self) -> i32 {
        self.class.unwrap_or(LIBUSB_HOTPLUG_MATCH_ANY)
    }

    pub(crate) fn get_events(&self) -> i32 {
        self.events.unwrap_or(LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED | LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT)
    }

    pub(crate) fn get_flags(&self) -> i32 {
        if self.enumerate {
            LIBUSB_HOTPLUG_ENUMERATE
        } else {
            0
        }
    }

    pub fn vendor(mut self, vendor: i32) -> Self {
        self.vendor = Some(vendor);
        self
    }

    pub fn product(mut self, product: i32) -> Self {
        self.product = Some(product);
        self
    }

    pub fn class(mut self, class: i32) -> Self {
        self.class = Some(class);
        self
    }

    pub fn arrived_only(mut self) -> Self {
        self.events = Some(LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED);
        self
    }

    pub fn left_only(mut self) -> Self {
        self.events = Some(LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT);
        self
    }

    pub fn enumerate(mut self) -> Self {
        self.enumerate = true;
        self
    }
}


