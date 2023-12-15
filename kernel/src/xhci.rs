use crate::pci::{
    scan_all_brute,
    // PciAddress,
    X64Access,
    PciHeader, EndpointHeader,
    Bar,
    capability::{
        PciCapability,
        MsiCapability,
        MultipleMessageSupport,
        TriggerMode,
    },
    device_type::{
        DeviceType,
        UsbType,
    },
};
use crate::allocator::global_allocator;

use core::ptr::NonNull;

pub use usb_xhci::controller::Controller;
pub use usb_xhci::class::{
    self,
    SupportedClassListeners
};

pub fn get_xhci_ep() -> Option<EndpointHeader> {
    scan_all_brute().find(|addr| {
        let (_, base, sub, ifce) = PciHeader::new(*addr).revision_and_class(&X64Access);

        DeviceType::from((base, sub)) == DeviceType::UsbController
        && UsbType::try_from(ifce).ok() == Some(UsbType::Xhci)
    }).and_then(|addr| {
        EndpointHeader::from_header(
            PciHeader::new(addr), &X64Access
        )
    })

    // let devices = Devices::<32>::scan().unwrap();
    // for dev in devices.as_slice() {
    //     log::debug!("{}.{}.{}.: vend {:04x}, class {:06x}, head {:02x}", dev.bus(), dev.slot_fun().0, dev.slot_fun().1, dev.vendor_id(), dev.class_code().code(), dev.header_type());
    // }

    // devices.as_slice().iter().find(|&dev| {
    //     dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x30)
    // }).map(|dev| *dev)
}

pub fn find_msi_cap(ep: &EndpointHeader) -> Option<MsiCapability> {
    ep.capabilities(&X64Access).find_map(|cap| {
        if let PciCapability::Msi(msi) = cap {
            Some(msi)
        } else { None }
    })
}

pub fn cfg_msi_fixed_dst(
    msi: MsiCapability,
    apic_base: NonNull<u8>,
    apic_id: u8,
    // we have `trigger_mode` here, but we will use `TriggerMode::Level`.
    // we have `delivery_mode` here, but we will only use `Fixed` (0).
    vector: u8,
    // num_vec_exp: MultipleMessageSupport,
    // we have MultipleMessageSupport, but we will use `Int1`(= 0)
) {
    // todo : this involves too many reads & writes. just save them and flush once!

    msi.set_multiple_message_enable(MultipleMessageSupport::Int1, &X64Access);

    let apic_base = apic_base.as_ptr() as usize;

    msi.set_message_info(
        (apic_base as u32) | ((apic_id as u32) << 12),
        vector,
        TriggerMode::LevelAssert,
        &X64Access,
    );

    msi.set_enabled(true, &X64Access);
}

pub fn read_mmio_base(ep: &EndpointHeader) -> Option<u64> {
    match ep.bar(0, &X64Access) {
        Some(Bar::Memory64 { address, .. }) => Some(address),
        Some(Bar::Memory32 { address, .. }) => Some(address as u64),
        _ => None
    }
}

pub fn setup_xhc_controller<L: SupportedClassListeners>(
    xhci_mmio_base: u64
) -> Option<Controller<'static, L>>
{
    let mut xhc = Controller::new(xhci_mmio_base, global_allocator());
    xhc.run();
    xhc.reconfigure_port();

    // after here, use `xhc.process_events()` to process events.

    Some(xhc)
}