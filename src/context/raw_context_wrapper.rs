use libusb::*;

pub struct RawContextWrapper {
    pub(crate) context: *mut libusb_context,
}

unsafe impl Sync for RawContextWrapper {}
unsafe impl Send for RawContextWrapper {}

impl Drop for RawContextWrapper {
    fn drop(&mut self) {
        unsafe {
            libusb_exit(self.context);
        }
    }
}

impl std::ops::Deref for RawContextWrapper {
    type Target = *mut libusb_context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
