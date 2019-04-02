use std::mem;

use libusb::*;

use crate::config_descriptor::{self, ConfigDescriptor};
use crate::context::Context;
use crate::device_descriptor::{self, DeviceDescriptor};
use crate::device_handle::{self, DeviceHandle};
use crate::error;
use crate::fields::{self, Speed};

/// A reference to a USB device.
pub struct Device {
    context: Context,
    device: *mut libusb_device,
}

impl Drop for Device {
    /// Releases the device reference.
    fn drop(&mut self) {
        unsafe {
            libusb_unref_device(self.device);
        }
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Device {
    /// Reads the device descriptor.
    pub fn device_descriptor(&self) -> error::Result<DeviceDescriptor> {
        let mut descriptor: libusb_device_descriptor = unsafe { mem::uninitialized() };

        // since libusb 1.0.16, this function always succeeds
        try_unsafe!(libusb_get_device_descriptor(self.device, &mut descriptor));

        Ok(device_descriptor::from_libusb(descriptor))
    }

    /// Reads a configuration descriptor.
    pub fn config_descriptor(&self, config_index: u8) -> error::Result<ConfigDescriptor> {
        let mut config: *const libusb_config_descriptor = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_get_config_descriptor(
            self.device,
            config_index,
            &mut config
        ));

        Ok(unsafe { config_descriptor::from_libusb(config) })
    }

    /// Reads the configuration descriptor for the current configuration.
    pub fn active_config_descriptor(&self) -> error::Result<ConfigDescriptor> {
        let mut config: *const libusb_config_descriptor = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_get_active_config_descriptor(
            self.device,
            &mut config
        ));

        Ok(unsafe { config_descriptor::from_libusb(config) })
    }

    /// Returns the number of the bus that the device is connected to.
    pub fn bus_number(&self) -> u8 {
        unsafe { libusb_get_bus_number(self.device) }
    }

    /// Returns the device's address on the bus that it's connected to.
    pub fn address(&self) -> u8 {
        unsafe { libusb_get_device_address(self.device) }
    }

    /// Returns the device's connection speed.
    pub fn speed(&self) -> Speed {
        fields::speed_from_libusb(unsafe { libusb_get_device_speed(self.device) })
    }

    /// Opens the device.
    pub fn open(&self) -> error::Result<DeviceHandle> {
        let mut handle: *mut libusb_device_handle = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_open(self.device, &mut handle));

        Ok(unsafe { device_handle::from_libusb(self.context.clone(), handle) })
    }
}

#[doc(hidden)]
pub unsafe fn from_libusb(context: Context, device: *mut libusb_device) -> Device {
    libusb_ref_device(device);

    Device {
        context: context,
        device: device,
    }
}
