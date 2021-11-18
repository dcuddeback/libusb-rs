use std::fmt;
use std::mem;
use libusb::*;

pub struct BosDescriptor {
    descriptor: *const libusb_bos_descriptor,
}

pub struct BosDevCapabilityDescriptor {
    addr: *const *const u8,
    descriptor: *const libusb_bos_dev_capability_descriptor,
}

pub struct Usb20ExtensionDescriptor {
    descriptor: *const libusb_usb_2_0_extension_descriptor,
}

pub struct SsUsbDescriptor {
    descriptor: *const libusb_ss_usb_device_capability_descriptor,
}

pub struct ContainerIdDescriptor {
    descriptor: *const libusb_container_id_descriptor,
}

impl Drop for BosDescriptor {
    fn drop(&mut self) {
        unsafe {
            libusb_free_bos_descriptor(self.descriptor as *mut libusb_bos_descriptor);
        }
    }
}

impl Drop for Usb20ExtensionDescriptor {
    fn drop(&mut self) {
        unsafe {
            libusb_free_usb_2_0_extension_descriptor(self.descriptor as *mut libusb_usb_2_0_extension_descriptor);
        }
    }
}

impl Drop for SsUsbDescriptor {
    fn drop(&mut self) {
        unsafe {
            libusb_free_ss_usb_device_capability_descriptor(self.descriptor as *mut libusb_ss_usb_device_capability_descriptor);
        }
    }
}

impl Drop for ContainerIdDescriptor {
    fn drop(&mut self) {
        unsafe {
            libusb_free_container_id_descriptor(self.descriptor as *mut libusb_container_id_descriptor);
        }
    }
}

/// BOS Descriptor
impl BosDescriptor {
    pub fn length(&self) -> u8 {
        unsafe {
            (*self.descriptor).bLength
        }
    }

    pub fn descriptor_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDescriptorType
        }
    }

    pub fn total_length(&self) -> u16 {
        unsafe {
            (*self.descriptor).wTotalLength
        }
    }

    pub fn num_device_caps(&self) -> u8 {
        unsafe {
            (*self.descriptor).bNumDeviceCaps
        }
    }

    pub fn dev_capability(&self) -> Vec<BosDevCapabilityDescriptor> {
        unsafe {
            let mut v: Vec<BosDevCapabilityDescriptor> = Vec::new();
            for i in 0..self.num_device_caps() {
                // 先转换成指针
                let point = std::ptr::addr_of!((*self.descriptor).dev_capability).offset(i as _);
                // 在将指针转换为 *const *const libusb_bos_dev_capability_descriptor，然后解引用两次
                let dev_cap = &(*(*(point as * const *const libusb_bos_dev_capability_descriptor)));
                let cap = from_libusb_bos_dev_capability_descriptor(point as *const *const u8, dev_cap);
                v.push(cap);
            }
            v
        }
    }
}

/// Device Capability Descriptor
impl BosDevCapabilityDescriptor {
    pub fn get_addr(&self) -> *const *const u8 {
        self.addr
    }

    pub fn length(&self) -> u8 {
        unsafe {
            (*self.descriptor).bLength
        }
    }

    pub fn descriptor_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDescriptorType
        }
    }

    pub fn dev_capability_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDevCapabilityType
        }
    }
}

/// USB 2.0 Extension Descriptor
impl Usb20ExtensionDescriptor {
    pub fn length(&self) -> u8 {
        unsafe {
            (*self.descriptor).bLength
        }
    }

    pub fn descriptor_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDescriptorType
        }
    }

    pub fn dev_capability_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDevCapabilityType
        }
    }

    pub fn attributes(&self) -> u32 {
        unsafe {
            (*self.descriptor).bmAttributes
        }
    }
}

/// SuperSpeed USB Descriptor
impl SsUsbDescriptor {
    pub fn length(&self) -> u8 {
        unsafe {
            (*self.descriptor).bLength
        }
    }

    pub fn descriptor_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDescriptorType
        }
    }

    pub fn dev_capability_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDevCapabilityType
        }
    }

    pub fn attributes(&self) -> u8 {
        unsafe {
            (*self.descriptor).bmAttributes
        }
    }

    pub fn speed_supported(&self) -> u16 {
        unsafe {
            (*self.descriptor).wSpeedSupported
        }
    }

    pub fn functionality_support(&self) -> u8 {
        unsafe {
            (*self.descriptor).bFunctionalitySupport
        }
    }

    pub fn u1_dev_exit_lat(&self) -> u8 {
        unsafe {
            (*self.descriptor).bU1DevExitLat
        }
    }

    pub fn u2_dev_exit_lat(&self) -> u8 {
        unsafe {
            (*self.descriptor).bU2DevExitLat
        }
    }
}

/// Container ID Descriptor
impl ContainerIdDescriptor {
    pub fn length(&self) -> u8 {
        unsafe {
            (*self.descriptor).bLength
        }
    }

    pub fn descriptor_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDescriptorType
        }
    }

    pub fn dev_capability_type(&self) -> u8 {
        unsafe {
            (*self.descriptor).bDevCapabilityType
        }
    }

    pub fn reserved(&self) -> u8 {
        unsafe {
            (*self.descriptor).bReserved
        }
    }

    pub fn container_id(&self) -> [u8; 16] {
        unsafe {
            (*self.descriptor).ContainerId
        }
    }
}

impl fmt::Debug for BosDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = f.debug_struct("BosDescriptor");

        let descriptor: &libusb_bos_descriptor = unsafe {
            mem::transmute(self.descriptor)
        };

        debug.field("bLength", &descriptor.bLength);
        debug.field("bDescriptorType", &descriptor.bDescriptorType);
        debug.field("wTotalLength", &descriptor.wTotalLength);
        debug.field("bNumDeviceCaps", &descriptor.bNumDeviceCaps);

        debug.finish()
    }
}

#[doc(hidden)]
pub fn from_libusb(bos: *const libusb_bos_descriptor) -> BosDescriptor {
    BosDescriptor { descriptor: bos }
}

#[doc(hidden)]
pub fn from_libusb_bos_dev_capability_descriptor(addr: *const *const u8, bos: *const libusb_bos_dev_capability_descriptor) -> BosDevCapabilityDescriptor {
    BosDevCapabilityDescriptor { addr: addr, descriptor: bos }
}

#[doc(hidden)]
pub fn from_libusb_usb_2_0_extension_descriptor(bos: *const libusb_usb_2_0_extension_descriptor) -> Usb20ExtensionDescriptor {
    Usb20ExtensionDescriptor { descriptor: bos }
}

#[doc(hidden)]
pub fn from_libusb_ss_usb_device_capability_descriptor(bos: *const libusb_ss_usb_device_capability_descriptor) -> SsUsbDescriptor {
    SsUsbDescriptor { descriptor: bos }
}

#[doc(hidden)]
pub fn from_libusb_container_id_descriptor(bos: *const libusb_container_id_descriptor) -> ContainerIdDescriptor {
    ContainerIdDescriptor { descriptor: bos }
}

#[cfg(test)]
mod test {
    use std::mem;

    // The Drop trait impl calls libusb_free_config_descriptor(), which would attempt to free
    // unallocated memory for a stack-allocated config descriptor. Allocating a config descriptor
    // is not a simple malloc()/free() inside libusb. Mimicking libusb's allocation would be
    // error-prone, difficult to maintain, and provide little benefit for the tests. It's easier to
    // use mem::forget() to prevent the Drop trait impl from running. The config descriptor passed
    // as `$config` should be stack-allocated to prevent memory leaks in the test suite.
    macro_rules! with_bos {
        ($name:ident : $bos:expr => $body:block) => {
            {
                let $name = unsafe { super::from_libusb(&$bos) };
                $body;
                mem::forget($name);
            }
        }
    }

    #[test]
    fn it_has_num_device_caps() {
        with_bos!(bos: bos_descriptor!(bNumDeviceCaps: 3) => {
            assert_eq!(3, bos.num_device_caps());
        });
    }
}
