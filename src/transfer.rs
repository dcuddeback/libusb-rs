use libusb::*;

use crate::{
    device_handle::{DeviceHandle, GetLibUsbDeviceHandle},
    error::{Error, Result},
};

pub enum TransferStatus {
    Completed,
    Error,
    Timeout,
    Cancelled,
    Stall,
    NoDevice,
    Overflow,
    Unknown,
}

impl TransferStatus {
    fn from_libusb(code: libc::c_int) -> Self {
        match code {
            0 => TransferStatus::Completed,
            1 => TransferStatus::Error,
            2 => TransferStatus::Timeout,
            3 => TransferStatus::Cancelled,
            4 => TransferStatus::Stall,
            5 => TransferStatus::NoDevice,
            6 => TransferStatus::Overflow,
            _ => TransferStatus::Unknown,
        }
    }
}

pub type TransferCallbackFunction = Option<Box<dyn FnMut(TransferStatus, i32)>>;

pub struct Transfer<'a> {
    transfer_handle: *mut libusb_transfer,
    callback: TransferCallbackFunction,
    _device_handle: &'a mut DeviceHandle<'a>,
}

impl<'a> Drop for Transfer<'a> {
    fn drop(&mut self) {
        unsafe {
            libusb_cancel_transfer(self.transfer_handle);
            libusb_free_transfer(self.transfer_handle);
        }
    }
}

impl<'a> Transfer<'a> {
    pub fn new(
        device_handle: &'a mut DeviceHandle<'a>,
        iso_packets: i32,
        flags: u8,
        endpoint: u8,
        transfer_type: u8,
        status: i32,
        timeout: u32,
        callback: TransferCallbackFunction,
    ) -> Result<Self> {
        let transfer_handle = unsafe { libusb_alloc_transfer(iso_packets) };
        if transfer_handle == std::ptr::null_mut() {
            return Err(Error::NoMem);
        }

        unsafe {
            (*transfer_handle).dev_handle = device_handle.get_lib_usb_handle();
            (*transfer_handle).flags = flags;
            (*transfer_handle).endpoint = endpoint;
            (*transfer_handle).transfer_type = transfer_type;
            (*transfer_handle).status = status;
            (*transfer_handle).timeout = timeout;
            (*transfer_handle).callback = libusb_transfer_callback_function;
        }

        let mut transfer = Self {
            transfer_handle,
            _device_handle: device_handle,
            callback,
        };

        unsafe {
            (*transfer_handle).user_data =
                std::mem::transmute::<&mut Transfer<'a>, *mut libc::c_void>(&mut transfer);
        }

        Ok(transfer)
    }

    pub fn submit_transfer(&mut self, data: &mut [u8]) -> Result<()> {
        unsafe {
            (*self.transfer_handle).buffer = data.as_mut_ptr();
            (*self.transfer_handle).length = data.len() as i32;
        }

        try_unsafe!(libusb_submit_transfer(self.transfer_handle));
        Ok(())
    }

    pub fn cancel_transfer(&mut self) -> Result<()> {
        try_unsafe!(libusb_cancel_transfer(self.transfer_handle));
        Ok(())
    }
}

extern "C" fn libusb_transfer_callback_function(transfer_handle: *mut libusb_transfer) {
    let transfer = unsafe {
        std::mem::transmute::<*mut libc::c_void, &mut Transfer<'_>>((*transfer_handle).user_data)
    };

    if let Some(ref mut callback) = transfer.callback {
        let status = unsafe { TransferStatus::from_libusb((*transfer_handle).status) };
        let actual_length = unsafe { (*transfer_handle).actual_length };

        callback(status, actual_length);
    }
}
