use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};

use libc::c_int;
use libusb::*;

use device::{self, Device};
use device_list::{self, DeviceList};
use device_handle::{self, DeviceHandle};
use error;
use event::HotPlugEvent;
use hotplug::{CallbackWrapper, HotplugFilter};

/// A `libusb` context.
pub struct Context {
    context: *mut libusb_context,
    cbs: Vec<Box<CallbackWrapper>>,
}

impl Drop for Context {
    /// Closes the `libusb` context.
    fn drop(&mut self) {
        eprintln!("Dropping a ctx");

        unsafe {
            for ref cb in &self.cbs {
                // TODO(richo) Deregister the callback
            }
            libusb_exit(self.context);
        }
    }
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Context {
    /// Opens a new `libusb` context.
    pub fn new() -> ::Result<Self> {
        let mut context = unsafe { mem::uninitialized() };

        try_unsafe!(libusb_init(&mut context));

        Ok(Context {
            context: context,
            cbs: vec![],
        })
    }

    /// Sets the log level of a `libusb` context.
    pub fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            libusb_set_debug(self.context, level.as_c_int());
        }
    }

    pub fn has_capability(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_HAS_CAPABILITY) != 0
        }
    }

    /// Tests whether the running `libusb` library supports hotplug.
    pub fn has_hotplug(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_HAS_HOTPLUG) != 0
        }
    }

    /// Tests whether the running `libusb` library has HID access.
    pub fn has_hid_access(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_HAS_HID_ACCESS) != 0
        }
    }

    /// Tests whether the running `libusb` library supports detaching the kernel driver.
    pub fn supports_detach_kernel_driver(&self) -> bool {
        unsafe {
            libusb_has_capability(LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) != 0
        }
    }

    /// Returns a list of the current USB devices. The context must outlive the device list.
    pub fn devices<'a>(&'a self) -> ::Result<DeviceList<'a>> {
        let mut list: *const *mut libusb_device = unsafe { mem::uninitialized() };

        let n = unsafe { libusb_get_device_list(self.context, &mut list) };

        if n < 0 {
            Err(error::from_libusb(n as c_int))
        }
        else {
            Ok(unsafe { device_list::from_libusb(self, list, n as usize) })
        }
    }


    /// Register a callback to fire when a device attached or removed.
    pub fn register_callback<F>(&mut self, filter: HotplugFilter, closure: F) -> ::Result<()>
    where F: Fn(&Device, HotPlugEvent) + 'static {
        let mut wrapper = Box::new(CallbackWrapper {
            closure: Box::new(closure),
            handle: 0,
        });
        let mut handle = 0;
        let res = unsafe { libusb_hotplug_register_callback(
            self.context,
            filter.get_events(),
            filter.get_flags(),
            filter.get_vendor(),
            filter.get_product(),
            filter.get_class(),
            invoke_callback,
            &mut *wrapper as *mut _ as *mut ::std::ffi::c_void,
            &mut handle)
        };
        if res != LIBUSB_SUCCESS {
            panic!("Couldn't setup callback");
        }
        wrapper.handle = handle;
        self.cbs.push(wrapper);
        Ok(())
    }

    pub fn handle_events(&self) {
        unsafe { libusb_handle_events(self.context) };
    }

    /// Convenience function to open a device by its vendor ID and product ID.
    ///
    /// This function is provided as a convenience for building prototypes without having to
    /// iterate a [`DeviceList`](struct.DeviceList.html). It is not meant for production
    /// applications.
    ///
    /// Returns a device handle for the first device found matching `vendor_id` and `product_id`.
    /// On error, or if the device could not be found, it returns `None`.
    pub fn open_device_with_vid_pid<'a>(&'a self, vendor_id: u16, product_id: u16) -> Option<DeviceHandle<'a>> {
        let handle = unsafe { libusb_open_device_with_vid_pid(self.context, vendor_id, product_id) };

        if handle.is_null() {
            None
        }
        else {
            Some(unsafe { device_handle::from_libusb(PhantomData, handle) })
        }
    }
}

extern "C" fn invoke_callback(_ctx: *mut libusb_context, device: *const libusb_device, event: i32, data: *mut std::ffi::c_void) -> i32 {
    match HotPlugEvent::from_i32(event) {
        Some(event) => {
            let device = ManuallyDrop::new(unsafe { device::from_libusb(PhantomData, device as *mut libusb_device) });

            let wrapper = data as *mut CallbackWrapper;

            unsafe { ((*wrapper).closure)(&device, event) };

            0
        },
        None => {
            // With no meaningful way to signal this error condition we simply don't dispatch the
            // call and return.
            return 0;
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
            LogLevel::None    => LIBUSB_LOG_LEVEL_NONE,
            LogLevel::Error   => LIBUSB_LOG_LEVEL_ERROR,
            LogLevel::Warning => LIBUSB_LOG_LEVEL_WARNING,
            LogLevel::Info    => LIBUSB_LOG_LEVEL_INFO,
            LogLevel::Debug   => LIBUSB_LOG_LEVEL_DEBUG,
        }
    }
}
