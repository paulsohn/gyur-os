extern crate alloc;

use core::alloc::Allocator;
use core::cell::RefCell;
use core::marker::PhantomData;
use alloc::alloc::Global;
use alloc::vec::Vec;
use alloc::boxed::Box;
// use alloc::rc::Rc;

use num_enum::{FromPrimitive, IntoPrimitive};

use crate::endpoint::{
    EndpointAddress,
    EndpointConfig,
};
use crate::descriptor::{
    DescriptorType,
    DescriptorIterator, Descriptor,
    DeviceDescriptorBody
};
use crate::setup::{requests, SetupRequest};

use crate::bus::USBBus;
use crate::class::{
    USBClass,
    SupportedClassListeners,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DeviceState {
    Attached,
    #[default]
    Default,
    Addressed,
    Configured,
    Suspend, // unused
}

pub struct Device<B, L, A = Global>
where
    B: USBBus,
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    buf: [u8; 256],

    device_desc: DeviceDescriptorBody,
    state: DeviceState,
    // class_drivers: Vec<RefCell<Box<dyn USBClass<B>, A>>, A>,
    // ep_configs: Vec<EndpointConfig, A>,

    allocator: A,
    _bus: PhantomData<B>,
    _listeners: PhantomData<L>,
}

impl<B, L, A> Device<B, L, A>
where
    B: USBBus,
    A: Allocator + Clone + 'static,
    L: SupportedClassListeners,
{
    /// Returns the device class. Would be zero if device desc is not set.
    // fn device_class(&self) -> u8 {
    //     self.desc.b_device_class
    // }

    /// This will be set true if device desc has been received (phase 1 ended)
    fn has_device_desc_received(&self) -> bool {
        self.device_desc != Default::default() // or desc should be Option<DeviceDescriptorBody>
    }

    /// This will be set true if the device is ready
    pub fn is_configured(&self) -> bool {
        self.state == DeviceState::Configured
    }

    pub fn new(allocator: A) -> Self {
        Self {
            buf: [0; 256],

            device_desc: Default::default(),
            state: DeviceState::Default,

            // class_drivers: Vec::new_in(allocator.clone()),
            // ep_configs: Vec::new_in(allocator.clone()),

            _listeners: PhantomData,
            _bus: PhantomData,
            allocator,
        }
    }

    fn on_device_desc_received(&mut self, buf: &[u8]) { // init phase 1
        let mut reader = DescriptorIterator::from_buf(buf);

        self.device_desc = match reader.next() {
            Some(Descriptor::Device(&desc)) => {
                assert_ne!(desc, Default::default(), "Received Trivial Device Descriptor"); // we want received desc to be nontrivial
                desc
            },
            _ => {
                panic!("Not a Device Descriptor");
            }
        };
    }

    /// Read a config descriptor and following interfaces.
    /// return the config value.
    fn on_config_desc_received(
        &mut self,
        buf: &[u8],
        class_drivers: &RefCell<Vec<Box<dyn USBClass, A>, A>>,
        ep_configs: &RefCell<Vec<EndpointConfig, A>>,
    ) -> u8 { // init phase 2
        use crate::class::new_class_from_interface;

        let mut reader = DescriptorIterator::from_buf(buf);

        let cfg = match reader.next() {
            Some(Descriptor::Configuration(&cfg)) => cfg,
            _ => {
                panic!("Not a configuration descriptor");
            }
        };

        // read interfaces and make class drivers.
        while let Some(Descriptor::Interface(&if_desc)) = reader.next() {
            if let Some(cls) = new_class_from_interface::<L, A>(
                if_desc.b_interface_class,
                if_desc.b_interface_sub_class,
                if_desc.b_interface_protocol,
                if_desc.b_interface_number,
                self.allocator.clone()
            ) {
                class_drivers.borrow_mut().push(cls);

                // read endpoints and extract configs.
                // in this case, some descriptors are not ep descriptors.
                let mut i = 0;
                while i < if_desc.b_num_endpoints {
                    if let Some(Descriptor::Endpoint(&ep_desc)) = reader.next() {
                        ep_configs.borrow_mut().push(
                            EndpointConfig::from_ep_desc(ep_desc)
                        );
                        i += 1;
                    }
                }
            }
        }
        // todo : support for device desc device class != 0 (CDC)

        cfg.b_configuration_value

        // if let Some(Descriptor::Configuration(&cfg)) = reader.next() {
        //     if self.device_desc.b_device_class == 0 {
        //         // read interfaces and make class drivers.
        //         while let Some(desc) = reader.next() {
        //             if let Descriptor::Interface(&if_desc) = desc {
        //                 if let Some(cls) = new_class::<L, A>(
        //                     if_desc.b_interface_class,
        //                     if_desc.b_interface_sub_class,
        //                     if_desc.b_interface_protocol,
        //                     if_desc,
        //                     self.allocator.clone()
        //                 ) {
        //                     class_drivers.borrow_mut().push(cls);
    
        //                     // read endpoints and extract configs.
        //                     // in this case, some descriptors are not ep descriptors.
        //                     let mut i = 0;
        //                     while i < if_desc.b_num_endpoints {
        //                         if let Some(Descriptor::Endpoint(&ep_desc)) = reader.next() {
        //                             ep_configs.borrow_mut().push(
        //                                 EndpointConfig::from_ep_desc(ep_desc)
        //                             );
        //                             i += 1;
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     } else {
        //         // use device class, subclass, protocol to make class driver.
        //         while let Some(desc) = reader.next() {
        //             if let Descriptor::Interface(&if_desc) = desc {
        //                 if let Some(cls) = new_class::<L, A>(
        //                     self.device_desc.b_device_class,
        //                     self.device_desc.b_device_sub_class,
        //                     self.device_desc.b_device_protocol,
        //                     if_desc,
        //                     self.allocator.clone()
        //                 ) {

        //                     class_drivers.borrow_mut().push(cls);
    
        //                     // read endpoints and extract configs.
        //                     // in this case, ep descriptors are the first `num_endpoints` descripters read.
        //                     let mut i = 0;
        //                     while i < if_desc.b_num_endpoints {
        //                         if let Some(Descriptor::Endpoint(&ep_desc)) = reader.next() {
        //                             ep_configs.borrow_mut().push(
        //                                 EndpointConfig::from_ep_desc(ep_desc)
        //                             );
        //                         }
        //                         i += 1;
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     cfg.b_configuration_value
        // } else {
        //     panic!("Not a Configuration Descriptor");
        // }
    }

    // `on_endpoints_configured`, `on_control_completed`, `on_normal_completed` :
    // same interface for usb class drivers.

    pub fn on_endpoints_configured(
        &self,
        bus: &B,
        class_drivers: &RefCell<Vec<Box<dyn USBClass, A>, A>>,
    ) {
        for cls in class_drivers.borrow_mut().iter_mut() {
            cls.on_endpoints_configured(bus);
        }
    }

    /// Start initialization.
    /// Following initialization processes will be done in `on_control_completed`.
    pub fn start_init(&mut self, bus: &B) {
        self.state = DeviceState::Default;

        // make get device descriptor call.
        bus.control_in(
            EndpointAddress::control(),
            requests::get_descriptor(DescriptorType::Device, 0, self.buf.len() as u16),
            &mut self.buf
        );
    }

    pub fn on_control_completed(
        &mut self,
        bus: &B,
        class_drivers: &RefCell<Vec<Box<dyn USBClass, A>, A>>,
        ep_configs: &RefCell<Vec<EndpointConfig, A>>,
        addr: EndpointAddress,
        req: SetupRequest,
        buf: &mut [u8],
    ) {
        // all error handlings should be done on each phase methods.

        if !self.has_device_desc_received() { // phase 1.
            self.on_device_desc_received(buf);
            
            // make get configuration descriptor call.
            // in this implementation, we will only use config index.
            bus.control_in(
                EndpointAddress::control(),
                requests::get_descriptor(DescriptorType::Configuration, 0, self.buf.len() as u16),
                &mut self.buf
            );
        } else if class_drivers.borrow().is_empty() { // phase 2.
            let cfg_value = self.on_config_desc_received(buf, class_drivers, ep_configs);

            // make set configuration call.
            bus.control_out(
                EndpointAddress::control(),
                requests::set_configuration(cfg_value as u16),
                self.buf.get(0..0).unwrap() // empty buffer
            );

            // This set configuration call should invoke each class drivers' set_endpoint.

            // // At this point, the device is considered configured.
            // self.state = DeviceState::Configured;
            
        }
        else if !self.is_configured() {
            for cls in class_drivers.borrow_mut().iter_mut() {
                cls.set_endpoint(ep_configs.borrow_mut().as_mut_slice());
                // cls.on_control_completed(bus, addr, req, buf);
            }

            // At this point, the device is considered configured.
            self.state = DeviceState::Configured;
        } else {
            // if this device has been configured, pass this event to class drivers and do nothing.
            for cls in class_drivers.borrow_mut().iter_mut() {
                cls.on_control_completed(bus, addr, req, buf);
            }
        }

        // for cls in class_drivers.borrow_mut().iter_mut() {
        //     cls.on_control_completed(bus, addr, req, buf);
        // }

    }

    pub fn on_normal_completed(
        &mut self,
        bus: &B,
        class_drivers: &RefCell<Vec<Box<dyn USBClass, A>, A>>,
        addr: EndpointAddress,
        buf: &mut [u8]
    ) {
        for cls in class_drivers.borrow_mut().iter_mut() {
            cls.on_normal_completed(bus, addr, buf);
        }
    }
}