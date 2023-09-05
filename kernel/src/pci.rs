use x86_64::instructions::port::Port;

const PCI_CONFIG_ADDR_PORT_NO: u16 = 0x0cf8;
const PCI_CONFIG_DATA_PORT_NO: u16 = 0x0cfc;

fn read_pci_config_data(bus: u8, dev: u8, fun: u8, offset: u8) -> u32 {
    let config_addr
        = 1u32         << 31     // enable bit
        | (bus as u32) << 16     // bus no(8bit)
        | (dev as u32) << 11     // dev no(5bit)
        | (fun as u32) <<  8     // fun no(3bit)
        | (offset & 0xfc) as u32;// offset(8bit), round off last 2bit
    unsafe {
        Port::<u32>::new(PCI_CONFIG_ADDR_PORT_NO).write( config_addr );
        Port::<u32>::new(PCI_CONFIG_DATA_PORT_NO).read()
    }
}

#[inline]
pub fn vendor_id(bus: u8, dev: u8, fun: u8) -> u16 {
    (read_pci_config_data(bus, dev, fun, 0x00) & 0xffff) as u16
}

#[inline]
pub fn device_id(bus: u8, dev: u8, fun: u8) -> u16 {
    (read_pci_config_data(bus, dev, fun, 0x00) >> 16) as u16
}

#[inline]
pub fn class_code(bus: u8, dev: u8, fun: u8) -> u32 {
    read_pci_config_data(bus, dev, fun, 0x08)
}

#[inline]
pub fn header_type(bus: u8, dev: u8, fun: u8) -> u8 {
    ((read_pci_config_data(bus, dev, fun, 0x0c) >> 16) & 0xff) as u8
}

#[inline]
pub fn bus_num(bus: u8, dev: u8, fun: u8) -> u32 {
    read_pci_config_data(bus, dev, fun, 0x18)
}

fn is_single_fun(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Device {
    pub bus: u8,
    pub dev: u8,
    pub fun: u8,
    pub header_type: u8, // @TODO why header_type is the only stored info?
}
pub enum Error {
    Full,
    Empty
}
pub type Result = core::result::Result<(), Error>;

pub struct Devices {
    store: [Device; 32],
    count: usize,
}

impl Devices {
    /// get a slice for devices.
    pub fn as_slice(&self) -> &[Device] {
        &self.store[..self.count]
    }

    pub fn scan() -> Option<Self> {
        let mut devices = Self { store: [Device::default(); 32], count: 0 };
        if devices.scan_all().is_ok() {
            Some(devices)
        } else { None }
    }

    fn scan_all(&mut self) -> Result {
        if is_single_fun(header_type(0, 0, 0)) {
            return self.scan_bus(0);
        }
        for fun in 0..8u8 { // 1..8u8 is said to be buggy
            if vendor_id(0, 0, fun) == 0xffff { continue; }
            self.scan_bus(fun)?; // fun as bus 
        }
        Ok(())
    }

    fn scan_bus(&mut self, bus: u8) -> Result {
        for dev in 0..32u8 {
            if vendor_id(bus, dev, 0) == 0xffff { continue; }
            self.scan_dev(bus, dev)?;
        }
        Ok(())
    }

    fn scan_dev(&mut self, bus: u8, dev: u8) -> Result {
        self.scan_fun(bus, dev, 0)?;
        if is_single_fun(header_type(bus, dev, 0)) {
            return Ok(());
        }
        for fun in 1..8u8 {
            if vendor_id(bus, dev, fun) == 0xffff { continue; }
            self.scan_fun(bus, dev, fun)?;
        }
        Ok(())
    }

    fn scan_fun(&mut self, bus: u8, dev: u8, fun: u8) -> Result {
        self.add_dev(bus, dev, fun)?;

        let class_code = class_code(bus, dev, fun);
        let base = ((class_code >> 24) & 0xff) as u8;
        let sub = ((class_code >> 16) & 0xff) as u8;

        if base == 0x06 && sub == 0x04 {
            let bus_num = bus_num(bus, dev, fun);
            let next_bus = ((bus_num >> 8) & 0xff) as u8;
            return self.scan_bus(next_bus);
        }
        Ok(())
    }

    fn add_dev(&mut self, bus: u8, dev: u8, fun: u8) -> Result {
        if self.count == 32 {
            return Err(Error::Full);
        }
        let header_type = header_type(bus, dev, fun);
        self.store[self.count] = Device { bus, dev, fun, header_type };
        self.count += 1;

        Ok(())
    }
}