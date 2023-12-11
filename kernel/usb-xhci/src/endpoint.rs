use num_enum::{FromPrimitive, IntoPrimitive};

use super::descriptor::EndpointDescriptorBody;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct EndpointAddress(u8);
impl EndpointAddress {
    /// Returns the control EP address, which is `0x80`.
    pub const fn control() -> Self {
        Self::from_parts(0, true)
    }

    /// Constructs a new EP address with the given index and direction.
    #[inline]
    pub const fn from_parts(index: usize, is_in: bool) -> Self {
        let i = index as u8;
        Self(
            if is_in { i | 0x80 } else { i }
        )
    }

    /// Constructs a new EP address from its byte representation.
    #[inline]
    pub const fn from_byte(b: u8) -> Self {
        Self(b)
    }

    /// Constructs a new EP address from its DCI representation.
    #[inline]
    pub const fn from_dci(b: u8) -> Self {
        // let i = b >> 1;
        // Self(
        //     if b & 1 == 1 { i | 0x80 } else { i }
        // )
        Self(b.rotate_right(1))
    }

    /// Returns the direction part of the EP address.
    /// True if the direction is IN.
    #[inline]
    pub const fn is_in(&self) -> bool {
        (self.0 & 0x80) != 0
    }

    /// Returns the index part of the EP address.
    #[inline]
    pub const fn index(&self) -> usize {
        (self.0 & 0x0F) as usize
    }

    /// Returns the byte representation of this address.
    #[inline]
    pub const fn byte(&self) -> u8 {
        self.0
    }

    /// Returns the DCI(Device Context Index) representation of this address.
    #[inline]
    pub const fn dci(&self) -> usize {
        // (self.index() << 1) | (self.is_in() as usize)
        self.0.rotate_left(1) as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EndpointType {
    #[num_enum(default)]
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

use xhci::context;
pub use context::EndpointType as EndpointDirectedType;

impl EndpointType {
    /// Returns a Directed Endpoint type for the given Endpoint type and its direction.
    pub fn with_dir(&self, is_in: bool) -> EndpointDirectedType {
        match self {
            EndpointType::Control => EndpointDirectedType::Control,
            _ => {
                // ((self as u8) + (if is_in { 4 } else { 0 })).into()
                use num_traits::FromPrimitive;
                EndpointDirectedType::from_u8(
                    (*self as u8) + (if is_in { 4 } else { 0 })
                ).unwrap()
            }
        }
        // match (self, is_in) {
        //     (EndpointType::Control, _) => EndpointDirectedType::Control,
        //     (EndpointType::Isochronous, false) => EndpointDirectedType::IsochOut,
        //     (EndpointType::Isochronous, true) => EndpointDirectedType::IsochIn,
        //     (EndpointType::Bulk, false) => EndpointDirectedType::BulkOut,
        //     (EndpointType::Bulk, true) => EndpointDirectedType::BulkIn,
        //     (EndpointType::Interrupt, false) => EndpointDirectedType::InterruptOut,
        //     (EndpointType::Interrupt, true) => EndpointDirectedType::InterruptIn,
        //     // _ => EndpointDirectedType::NotValid,
        // }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EndpointConfig {
    pub addr: EndpointAddress,
    ep_type: EndpointType,
    pub max_packet_size: u16,
    pub interval: u8,
}
impl EndpointConfig {
    pub fn from_ep_desc(ep_desc: EndpointDescriptorBody) -> Self {
        Self {
            addr: EndpointAddress::from_byte(ep_desc.b_endpoint_address),
            ep_type: ep_desc.transfer_type(),
            max_packet_size: ep_desc.w_max_packet_size,
            interval: ep_desc.b_interval,
        }
    }

    pub fn ep_type(&self) -> EndpointType {
        self.ep_type
    }

    pub fn ep_type_with_dir(&self) -> EndpointDirectedType {
        self.ep_type.with_dir(self.addr.is_in())
    }
}