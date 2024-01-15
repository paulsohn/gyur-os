use bit_field::BitField;
use x86_64::instructions::port::Port;

pub use pci_types::{
    PciAddress,
    DwordAccessMethod,
    PciHeader, EndpointHeader,
    Bar,
};
pub use pci_types::capability;
pub use pci_types::device_type;
pub use pci_types::accessor::{
    DwordAccessor,
    AccessorTrait,
};
pub use pci_types::map_field as acc_map_field;

use pci_types::dwords::HeaderTypeDword;

#[derive(Clone, Copy, Debug)]
pub struct LegacyPortAccessMethod;
impl LegacyPortAccessMethod {
    const CFG_ADDR_PORT_NO: u16 = 0x0cf8;
    const CFG_DATA_PORT_NO: u16 = 0x0cfc;
    const fn cfg_addr_port() -> Port<u32> {
        Port::new(Self::CFG_ADDR_PORT_NO)
    }
    const fn cfg_data_port() -> Port<u32> {
        Port::new(Self::CFG_DATA_PORT_NO)
    }

    fn addr_value(addr: PciAddress, offset: u16) -> u32 {
        // segment groups are unsupported currently. assume group 0.
        assert!(offset % 4 == 0);
        assert!(offset <= u8::MAX as u16, "u16 offsets are unsupported");
        *0u32.set_bit(31, true) // enable bit
            .set_bits(16..=23, addr.bus() as u32)
            .set_bits(11..=15, addr.slot() as u32)
            .set_bits(8..=10, addr.function() as u32)
            .set_bits(0..=7, offset as u32)
    }
}
impl DwordAccessMethod for LegacyPortAccessMethod {
    unsafe fn read_dword(&self, address: PciAddress, offset: u16) -> u32 {
        Self::cfg_addr_port().write(
            Self::addr_value(address, offset)
        );
        Self::cfg_data_port().read()
    }

    unsafe fn write_dword(&self, address: PciAddress, offset: u16, value: u32) {
        Self::cfg_addr_port().write(
            Self::addr_value(address, offset)
        );
        Self::cfg_data_port().write(value);
    }

    fn function_exists(&self, address: PciAddress) -> bool {
        unsafe {
            self.read_dword(address, 0x00) != 0xffffffff
        }
    }

    fn has_multiple_functions(&self, address: PciAddress) -> bool {
        unsafe {
            core::mem::transmute::<u32, HeaderTypeDword>(
                self.read_dword(address, 0x0C)
            ).has_multiple_functions()
        }
    }
}

/// Scan all by Enumerating all 65536 (bus, dev, fun) triples.
/// Alternatively we can use DFS though.
pub fn scan_all_brute() -> impl Iterator<Item = PciAddress> {
    use itertools::Itertools;

    Itertools::cartesian_product(
        0..=0xffu8, // bus
        0..=31u8, // dev
    ).flat_map(|(bus, dev)| {
        // segment is unused, set to 0.

        let addr0 = PciAddress::new(0, bus, dev, 0x00);

        let opt0 = if LegacyPortAccessMethod.function_exists(addr0) {
            Some(addr0)
        } else { None };

        [opt0].into_iter().chain(
            if LegacyPortAccessMethod.has_multiple_functions(addr0) {
                core::array::from_fn::<u8, 7, _>(|i| (i as u8 + 1)).map(|fun| {
                    let addr = PciAddress::new(0, bus, dev, fun);
                    if LegacyPortAccessMethod.function_exists(addr) {
                        Some(addr)
                    } else { None }
                }).into_iter()
            } else {
                [None; 7].into_iter()
            }
        )
    }).flatten()
}

