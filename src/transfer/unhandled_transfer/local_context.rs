use libusb::*;
use std::marker::PhantomData;

use crate::Context;

#[derive(Copy, Clone)]
pub struct LocalContext<'a> {
    _parent_context: PhantomData<&'a Context>,
    pub raw_context: *mut libusb_context,
}
unsafe impl<'a> Sync for LocalContext<'a> {}
unsafe impl<'a> Send for LocalContext<'a> {}

impl<'a> LocalContext<'a> {
    pub fn new(raw_context: *mut libusb_context, _context: PhantomData<&'a Context>) -> Self {
        Self {
            raw_context,
            _parent_context: PhantomData,
        }
    }
}

impl<'a> std::ops::Deref for LocalContext<'a> {
    type Target = *mut libusb_context;

    fn deref(&self) -> &Self::Target {
        &self.raw_context
    }
}
