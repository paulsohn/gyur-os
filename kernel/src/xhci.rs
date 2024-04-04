extern crate alloc;

use crate::pci::{
    scan_all_brute,
    // PciAddress,    
    LegacyPortAccessMethod,
    // DwordAccessMethod,
    DwordAccessor, AccessorTrait,
    PciHeader, EndpointHeader,
    Bar,
    capability::CapabilityHeader,
    capability::msi::{
        MsiCapabilityInfo,
        // MessageControl,
        MessageData,
        MultipleMessageSupport,
        TriggerMode,
    },
    device_type::{
        DeviceType,
        UsbType,
    },
    acc_map_field,
};

use core::ptr::NonNull;
use core::alloc::Allocator;

pub use usb_xhci::controller::Controller;
pub use usb_xhci::class::{
    self,
    SupportedClassListeners
};

pub fn get_xhci_ep_acc<'a>()
-> Option<impl AccessorTrait<'a, LegacyPortAccessMethod, EndpointHeader>>
{
    scan_all_brute()
        .find(|&addr| {
            let accessor = DwordAccessor::<'_, _, PciHeader>::new(addr, 0, LegacyPortAccessMethod);

            let revcc = acc_map_field!(accessor.revcc).read();

            revcc.device_type() == DeviceType::UsbController
            && UsbType::try_from(revcc.interface) == Ok(UsbType::Xhci)
        }).map(|addr| {
            DwordAccessor::<'_, _, EndpointHeader>::new(addr, 0, LegacyPortAccessMethod)
        })

    // devices.as_slice().iter().find(|&dev| {
    //     dev.class_code().match_base_sub_interface(0x0c, 0x03, 0x30)
    // }).map(|dev| *dev)
}

pub fn find_msi_cap_acc<'a>(
    ep_acc: &impl AccessorTrait<'a, LegacyPortAccessMethod, EndpointHeader>
) -> Option<impl AccessorTrait<'a, LegacyPortAccessMethod, CapabilityHeader>>
{
    EndpointHeader::capabilities(ep_acc)
        .find(|cap| {
            let _c: CapabilityHeader = cap.read();

            // log::info!("Cap type {}, offset {}", c.id, cap.start_offset());

            MsiCapabilityInfo::msi_cap_type(cap) != 0
        })
}

pub fn cfg_msi_fixed_dst<'a>(
    msi_cap_header_acc: &impl AccessorTrait<'a, LegacyPortAccessMethod, CapabilityHeader>,
    apic_base: NonNull<u8>,
    apic_id: u8,
    // we have `trigger_mode` here, but we will use `TriggerMode::Level`.
    // we have `delivery_mode` here, but we will only use `Fixed` (0).
    vector: u8,
    // num_vec_exp: MultipleMessageSupport,
    // we have MultipleMessageSupport, but we will use `Int1`(= 0)
) {
    MsiCapabilityInfo::update_info(
        msi_cap_header_acc,
        |mut info| {
            {
                let control = &mut info.header.extension;
                control.set_msi_enable();
                control.set_multiple_message_enable(
                    MultipleMessageSupport::Int1
                );
            }
            info.addr[0] = (apic_base.as_ptr() as usize as u32) | ((apic_id as u32) << 12);
            info.data = MessageData::new(vector, TriggerMode::LevelAssert);
            info
        }
    );
}

pub fn read_mmio_base<'a>(
    ep_acc: &impl AccessorTrait<'a, LegacyPortAccessMethod, EndpointHeader>
) -> Option<u64> {
    // match EndpointHeader::bar(ep_acc, 0) {
    match EndpointHeader::bar_base_only(ep_acc, 0) {
        Some(Bar::Memory64 { address, .. }) => Some(address),
        Some(Bar::Memory32 { address, .. }) => Some(address as u64),
        _ => None
    }
}

pub fn setup_xhc_controller<'a, L: SupportedClassListeners, A: Allocator + Clone>(
    xhci_mmio_base: u64,
    allocator: A,
) -> Option<Controller<'a, L, A>>
{
    let mut xhc = Controller::new(xhci_mmio_base, allocator);
    xhc.run();
    xhc.reconfigure_port();

    // after here, use `xhc.process_events()` to process events.

    Some(xhc)
}