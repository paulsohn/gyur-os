use bit_field::BitField;
// use num_traits::FromPrimitive;
// use num_derive::FromPrimitive;
use num_enum::{FromPrimitive, IntoPrimitive};

use xhci::ring::trb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RequestType {
    Standard = 0,
    Class = 1,
    Vendor = 2,
    #[num_enum(default)]
    Reserved = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Recipient {
    Device = 0,
    Interface = 1,
    Endpoint = 2,
    Other = 3,
    #[num_enum(default)]
    Reserved = 4,
}

pub mod request_code {
    macro_rules! def {
        ($id:ident, $v:expr) => {
            pub const $id: u8 = $v;
        }
    }

    def!(GET_STATUS, 0);
    def!(CLEAR_FEATURE, 1);
    def!(SET_FEATURE, 3);
    def!(SET_ADDRESS, 5);
    def!(GET_DESCRIPTOR, 6);
    def!(SET_DESCRIPTOR, 7);
    def!(GET_CONFIGURATION, 8);
    def!(SET_CONFIGURATION, 9);
    def!(GET_INTERFACE, 10);
    def!(SET_INTERFACE, 11);
    def!(SYNCH_FRAME, 12);
    def!(SET_ENCRYPTION, 13);
    def!(GET_ENCRYPTION, 14);
    def!(SET_HANDSHAKE, 15);
    def!(GET_HANDSHAKE, 16);
    def!(SET_CONNECTION, 17);
    def!(SET_SECURITY_DATA, 18);
    def!(GET_SECURITY_DATA, 19);
    def!(SET_W_USB_DATA, 20);
    def!(LOOPBACK_DATA_WRITE, 21);
    def!(LOOPBACK_DATA_READ, 22);
    def!(SET_INTERFACE_OS, 23);
    def!(SET_SEL, 48);
    def!(SET_ISOCH_DELAY, 49);

    // HID class specific request values
    def!(GET_REPORT, 1);
    def!(SET_PROTOCOL, 11);

    // CDC class specific request values
    def!(SET_LINE_CODING, 32);
    def!(GET_LINE_CODING, 33);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(C, packed)]
pub struct SetupRequest {
    request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupRequest {
    pub fn new(is_in: bool, req_ty: RequestType, recipient: Recipient, req_code: u8, value: u16, index: u16, length: u16) -> Self {
        let mut req = Self {
            request_type: 0,
            request: req_code,
            value,
            index,
            length
        };
        req.set_is_in(is_in)
            .set_request_type(req_ty)
            .set_recipient(recipient);
        req
    }

    pub fn is_in(&self) -> bool {
        self.request_type.get_bit(7)
    }

    pub fn set_is_in(&mut self, is_in: bool) -> &mut Self {
        self.request_type.set_bit(7, is_in);
        self
    }

    pub fn request_type(&self) -> RequestType {
        RequestType::from(
            self.request_type.get_bits(5..=6)
        )
    }

    pub fn set_request_type(&mut self, req_ty: RequestType) -> &mut Self {
        self.request_type.set_bits(5..=6, req_ty.into());
        self
    }

    pub fn recipient(&self) -> Recipient {
        Recipient::from(
            self.request_type.get_bits(0..=4)
        )
    }

    pub fn set_recipient(&mut self, recipient: Recipient) -> &mut Self {
        self.request_type.set_bits(0..=4, recipient.into());
        self
    }
}

/// Correspondence between setup request and setup stage trb.
impl SetupRequest {
    /// Extract setup request from setup stage TRB.
    pub fn from_setup_stage_trb(trb: trb::transfer::SetupStage) -> Self {
        // can just copy first 8 bytes from this trb.
        Self {
            request_type: trb.request_type(),
            request: trb.request(),
            value: trb.value(),
            index: trb.index(),
            length: trb.length()
        }
    }

    /// Transform setup request into setup stage TRB.
    /// 
    /// Note that some fields, including transfer length and transfer type
    /// should be manually set after calling this method.
    pub fn into_setup_stage_trb(&self) -> trb::transfer::SetupStage {
        *trb::transfer::SetupStage::new()
            .set_request_type(self.request_type)
            .set_request(self.request)
            .set_value(self.value)
            .set_index(self.index)
            .set_length(self.length)
    }
}

pub mod requests {
    //! Pre-defined setup requests.
    
    use super::*;
    use crate::descriptor::DescriptorType;

    pub fn get_descriptor(desc_type: DescriptorType, desc_index: u8, len: u16) -> SetupRequest {
        SetupRequest::new(
            true,
            RequestType::Standard,
            Recipient::Device,
            request_code::GET_DESCRIPTOR,
            ((u8::from(desc_type) as u16) << 8) | (desc_index as u16),
            0,
            len
        )
    }

    pub fn set_configuration(cfg_value: u16) -> SetupRequest {
        SetupRequest::new(
            false,
            RequestType::Standard,
            Recipient::Device,
            request_code::SET_CONFIGURATION,
            cfg_value,
            0,
            0
        )
    }

    pub fn set_protocol(if_index: u16) -> SetupRequest {
        SetupRequest::new(
            false,
            RequestType::Class,
            Recipient::Interface,
            request_code::SET_PROTOCOL,
            0,
            if_index,
            0
        )
    }
}