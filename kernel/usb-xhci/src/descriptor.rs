use num_enum::{FromPrimitive, IntoPrimitive};
use bit_field::BitField;

use super::endpoint::EndpointType;

/// USB descriptor structure.
/// Note that this is only for illustrating concrete data structure.
/// For real use, consider [`Descriptor`] instead.
#[deprecated]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct DescriptorStruct<BODY> {
    header: DescriptorHeader,
    body: BODY,
}

/// USB descriptor header common to all descriptor types.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct DescriptorHeader {
    pub b_length: u8,
    pub b_descriptor_type: u8, // DescriptorType
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive,)]
#[repr(u8)]
pub enum DescriptorType {
    Device = 0x01,
    Configuration = 0x02,
    Interface = 0x03,
    Endpoint = 0x05,
    HID = 0x21,
    #[num_enum(catch_all)]
    Unsupported(u8),
}

#[derive(Clone, Copy, Debug)]
pub enum Descriptor<'a> {
    Device(&'a DeviceDescriptorBody),
    Configuration(&'a ConfigurationDescriptorBody),
    Interface(&'a InterfaceDescriptorBody),
    Endpoint(&'a EndpointDescriptorBody),
    Unsupported,
}

macro_rules! desc_body_cast {
    ($vari:ident, $ty:ident, $body_ptr:expr) => {
        Descriptor::$vari(unsafe {
            core::mem::transmute::<_, &$ty>($body_ptr) as _
        })
    }
}

pub struct DescriptorIterator<'a> {
    buf: &'a [u8],
    idx: usize,
}
impl<'a> DescriptorIterator<'a> {
    pub fn from_buf(buf: &'a [u8]) -> Self {
        Self {
            buf,
            idx: 0
        }
    }
}
impl<'a> Iterator for DescriptorIterator<'a> {
    type Item = Descriptor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.buf.len() {
            None
        } else {
            let len = self.buf[self.idx] as usize;
            let ty = self.buf[self.idx+1].into();

            let desc_body_ptr = unsafe {
                self.buf.as_ptr().add(self.idx+2)
            };
            self.idx += len;

            Some(
                match ty {
                    // 0x01 => Descriptor::Device(unsafe {
                    //     core::mem::transmute::<_, &DeviceDescriptorBody>(desc_body_ptr) as _
                    // }),
                    DescriptorType::Device => desc_body_cast!(Device, DeviceDescriptorBody, desc_body_ptr),
                    DescriptorType::Configuration => desc_body_cast!(Configuration, ConfigurationDescriptorBody, desc_body_ptr),
                    DescriptorType::Interface => desc_body_cast!(Interface, InterfaceDescriptorBody, desc_body_ptr),
                    DescriptorType::Endpoint => desc_body_cast!(Endpoint, EndpointDescriptorBody, desc_body_ptr),
                    _ => Descriptor::Unsupported,
                }
            )
        }
    }
}


/// USB device descriptor body, excluding header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C, packed)]
pub struct DeviceDescriptorBody {
    pub bcd_usb: u16,
    pub b_device_class: u8,
    pub b_device_sub_class: u8,
    pub b_device_protocol: u8,
    pub b_max_packet_size_0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub i_manufacturer: u8,
    pub i_product: u8,
    pub i_serial_number: u8,
    pub b_num_configurations: u8,
}

/// USB configuration descriptor body, excluding header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C, packed)]
pub struct ConfigurationDescriptorBody {
    pub w_total_length: u16,
    pub b_num_interfaces: u8,
    pub b_configuration_value: u8,
    pub i_configuration: u8,
    bm_attributes: u8,
    pub b_max_power: u8,
}

impl ConfigurationDescriptorBody {
    /// Returns Remote Wakeup bit of the attribute field.
    pub fn remote_wakeup(&self) -> bool {
        self.bm_attributes.get_bit(5)
    }

    /// Returns Self Powered bit of the attribute field.
    pub fn self_powered(&self) -> bool {
        self.bm_attributes.get_bit(6)
    }
}

/// USB interface descriptor body, excluding header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C, packed)]
pub struct InterfaceDescriptorBody {
    pub b_interface_number: u8,
    pub b_alternate_setting: u8,
    pub b_num_endpoints: u8,
    pub b_interface_class: u8,
    pub b_interface_sub_class: u8,
    pub b_interface_protocol: u8,
    pub i_interface: u8,
}

/// USB endpoint descriptor body, excluding header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C, packed)]
pub struct EndpointDescriptorBody {
    pub b_endpoint_address: u8,
    bm_attributes: u8,
    pub w_max_packet_size: u16,
    pub b_interval: u8,
}

impl EndpointDescriptorBody {
    pub fn transfer_type(&self) -> EndpointType {
        self.bm_attributes.get_bits(0..=1).into()
    }

    pub fn sync_type(&self) -> u8 {
        self.bm_attributes.get_bits(2..=3)
    }

    pub fn usage_type(&self) -> u8 {
        self.bm_attributes.get_bits(4..=5)
    }
}