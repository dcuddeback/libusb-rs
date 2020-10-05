use libusb::*;
use std::marker::PhantomData;

use crate::{
    device_handle::{DeviceHandle, GetLibUsbDeviceHandle},
    error::{Error, Result},
};

#[derive(Clone)]
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
    pub fn to_libusb_result(&self) -> Result<()> {
        match self {
            Self::Completed => Ok(()),
            Self::Timeout => Err(Error::Timeout),
            Self::Stall => Err(Error::Pipe),
            Self::NoDevice => Err(Error::NoDevice),
            Self::Overflow => Err(Error::Overflow),
            Self::Error | Self::Cancelled => Err(Error::Io),
            _ => Err(Error::Other),
        }
    }
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

pub struct Transfer<'a, 'b> {
    transfer_handle: *mut libusb_transfer,
    callback: TransferCallbackFunction,
    _device_handle: PhantomData<&'b DeviceHandle<'a>>,
}

impl<'a, 'b> Drop for Transfer<'a, 'b> {
    fn drop(&mut self) {
        unsafe {
            libusb_cancel_transfer(self.transfer_handle);
            libusb_free_transfer(self.transfer_handle);
        }
    }
}

impl<'a, 'b> Transfer<'a, 'b> {
    pub fn new(device_handle: &'b mut DeviceHandle<'a>, iso_packets: i32) -> Result<Self> {
        let transfer_handle = Self::allocate_trhansfer_handle(iso_packets)?;

        unsafe {
            (*transfer_handle).dev_handle = device_handle.get_lib_usb_handle();
            (*transfer_handle).callback = libusb_transfer_callback_function;
        }

        let mut transfer = Self {
            transfer_handle,
            _device_handle: PhantomData,
            callback: None,
        };

        unsafe {
            (*transfer_handle).user_data =
                std::mem::transmute::<&mut Transfer<'a, 'b>, *mut libc::c_void>(&mut transfer);
        }

        Ok(transfer)
    }

    pub fn set_flags(&mut self, flags: u8) -> Result<()> {
        unsafe {
            (*self.transfer_handle).flags = flags;
        };
        Ok(())
    }

    pub fn set_endpoint(&mut self, endpoint: u8) -> Result<()> {
        unsafe {
            (*self.transfer_handle).endpoint = endpoint;
        };
        Ok(())
    }

    pub fn set_transfer_type(&mut self, transfer_type: u8) -> Result<()> {
        unsafe {
            (*self.transfer_handle).transfer_type = transfer_type;
        };
        Ok(())
    }

    pub fn set_status(&mut self, status: i32) -> Result<()> {
        unsafe {
            (*self.transfer_handle).status = status;
        };
        Ok(())
    }

    pub fn set_timeout(&mut self, timeout: u32) -> Result<()> {
        unsafe {
            (*self.transfer_handle).timeout = timeout;
        };
        Ok(())
    }

    pub fn set_callback(&mut self, callback: TransferCallbackFunction) -> Result<()> {
        self.callback = callback;
        Ok(())
    }

    pub fn append_setup_packet_and_submit_transfer(
        &mut self,
        data: &mut [u8],
        bm_request_type: u8,
        b_request: u8,
        w_value: u16,
        w_index: u16,
    ) -> Result<()> {
        let setup_packet = Self::create_setup_packet(
            bm_request_type,
            b_request,
            w_value,
            w_index,
            data.len() as u16,
        )?;
        let mut data = setup_packet
            .into_iter()
            .chain(data.iter().copied())
            .collect::<Vec<u8>>();
        self.submit_transfer(data.as_mut())?;
        Ok(())
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

    pub fn create_setup_packet(
        bm_request_type: u8,
        b_request: u8,
        w_value: u16,
        w_index: u16,
        data_size: u16,
    ) -> Result<Vec<u8>> {
        let mut setup_packet = vec![bm_request_type, b_request];
        setup_packet.extend(w_value.to_le_bytes().iter());
        setup_packet.extend(w_index.to_le_bytes().iter());
        setup_packet.extend(data_size.to_le_bytes().iter());

        Ok(setup_packet)
    }

    fn allocate_trhansfer_handle(iso_packets: i32) -> Result<*mut libusb_transfer> {
        let transfer_handle = unsafe { libusb_alloc_transfer(iso_packets) };
        if transfer_handle == std::ptr::null_mut() {
            return Err(Error::NoMem);
        }

        Ok(transfer_handle)
    }
}

extern "C" fn libusb_transfer_callback_function(transfer_handle: *mut libusb_transfer) {
    let transfer = unsafe {
        std::mem::transmute::<*mut libc::c_void, &mut Transfer<'_, '_>>(
            (*transfer_handle).user_data,
        )
    };

    if let Some(ref mut callback) = transfer.callback {
        let status = unsafe { TransferStatus::from_libusb((*transfer_handle).status) };
        let actual_length = unsafe { (*transfer_handle).actual_length };

        callback(status, actual_length);
    }
}
