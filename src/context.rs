mod raw_context_wrapper;
use self::raw_context_wrapper::RawContextWrapper;

use std::{mem::MaybeUninit, sync::Arc};

use libc::c_int;
use libusb::*;

use device_handle::{self, DeviceHandle};
use device_list::{self, DeviceList};
use error;

/// A `libusb` context.
pub struct Context {
    pub(crate) context: Arc<RawContextWrapper>,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
        }
    }
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Context {
    /// Opens a new `libusb` context.
    pub fn new() -> ::Result<Self> {
        let mut context = unsafe { MaybeUninit::uninit().assume_init() };
        try_unsafe!(libusb_init(&mut context));
        Ok(Self {
            context: Arc::new(RawContextWrapper { context }),
        })
    }

    /// Sets the log level of a `libusb` context.
    pub fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            libusb_set_debug(**self.context, level.as_c_int());
        }
    }

    pub fn has_capability(&self) -> bool {
        unsafe { libusb_has_capability(LIBUSB_CAP_HAS_CAPABILITY) != 0 }
    }

    /// Tests whether the running `libusb` library supports hotplug.
    pub fn has_hotplug(&self) -> bool {
        unsafe { libusb_has_capability(LIBUSB_CAP_HAS_HOTPLUG) != 0 }
    }

    /// Tests whether the running `libusb` library has HID access.
    pub fn has_hid_access(&self) -> bool {
        unsafe { libusb_has_capability(LIBUSB_CAP_HAS_HID_ACCESS) != 0 }
    }

    /// Tests whether the running `libusb` library supports detaching the kernel driver.
    pub fn supports_detach_kernel_driver(&self) -> bool {
        unsafe { libusb_has_capability(LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) != 0 }
    }

    /// Returns a list of the current USB devices. The context must outlive the device list.
    pub fn devices(&self) -> ::Result<DeviceList> {
        let mut list: *const *mut libusb_device = unsafe { MaybeUninit::uninit().assume_init() };

        let n = unsafe { libusb_get_device_list(**self.context, &mut list) };

        if n < 0 {
            Err(error::from_libusb(n as c_int))
        } else {
            Ok(unsafe { device_list::from_libusb(self.clone(), list, n as usize) })
        }
    }

    /// Convenience function to open a device by its vendor ID and product ID.
    ///
    /// This function is provided as a convenience for building prototypes without having to
    /// iterate a [`DeviceList`](struct.DeviceList.html). It is not meant for production
    /// applications.
    ///
    /// Returns a device handle for the first device found matching `vendor_id` and `product_id`.
    /// On error, or if the device could not be found, it returns `None`.
    pub fn open_device_with_vid_pid(
        &self,
        vendor_id: u16,
        product_id: u16,
    ) -> Option<DeviceHandle> {
        let handle =
            unsafe { libusb_open_device_with_vid_pid(**self.context, vendor_id, product_id) };

        if handle.is_null() {
            None
        } else {
            Some(unsafe { device_handle::from_libusb(self.clone(), handle) })
        }
    }
}

/// Library logging levels.
pub enum LogLevel {
    /// No messages are printed by `libusb` (default).
    None,

    /// Error messages printed to `stderr`.
    Error,

    /// Warning and error messages are printed to `stderr`.
    Warning,

    /// Informational messages are printed to `stdout`. Warnings and error messages are printed to
    /// `stderr`.
    Info,

    /// Debug and informational messages are printed to `stdout`. Warnings and error messages are
    /// printed to `stderr`.
    Debug,
}

impl LogLevel {
    fn as_c_int(&self) -> c_int {
        match *self {
            LogLevel::None => LIBUSB_LOG_LEVEL_NONE,
            LogLevel::Error => LIBUSB_LOG_LEVEL_ERROR,
            LogLevel::Warning => LIBUSB_LOG_LEVEL_WARNING,
            LogLevel::Info => LIBUSB_LOG_LEVEL_INFO,
            LogLevel::Debug => LIBUSB_LOG_LEVEL_DEBUG,
        }
    }
}
