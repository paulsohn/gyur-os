use x86_64::instructions::port::Port;

const CFG_ADDR_PORT_NO: u16 = 0x0cf8;
const CFG_DATA_PORT_NO: u16 = 0x0cfc;

/// PCI device.
/// Internally a device is identified by its config header address.
/// Config info abount this device may be obtained via Port I/O.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Device(u32);

impl Device {
    /// Create config space header address from (bus, dev, fun) triple.
    /// The enable bit is set by default.
    pub const fn from_bdf(bus: u8, dev: u8, fun: u8) -> Self {
        //   1u32         << 31     // enable bit
        // | (bus as u32) << 16     // bus no(8bit)
        // | (dev as u32) << 11     // dev no(5bit)
        // | (fun as u32) <<  8     // fun no(3bit)
        // | (offset & 0xfc) as u32;// offset(8bit), round off last 2bit
        Self(
            // u32::from_le_bytes([0, (dev << 3) | fun, bus, 0x80])
            1u32           << 31     // enable bit
            | (bus as u32) << 16     // bus no(8bit)
            | (dev as u32) << 11     // dev no(5bit)
            | (fun as u32) <<  8     // fun no(3bit)
        )
    }

    /// returns `bus` no. of this configuration.
    pub const fn bus(&self) -> u8 {
        // ((self.0 & 0xff0000) >> 16) as u8
        u32::to_le_bytes(self.0)[2]
    }

    /// returns `(dev, fun)` pair of this configuration.
    pub const fn dev_fun(&self) -> (u8, u8) {
        // let dev_fun = ((self.0 & 0xff00) >> 16) as u8;
        let dev_fun = u32::to_le_bytes(self.0)[1];
        ((dev_fun & 0xf8) >> 3, dev_fun & 0x07)
    }

    /// Read 4 bytes from the address plus offset specified.
    fn read_offset(&self, offset: u8) -> u32 {
        let addr  = self.0 | ((offset & 0xfc) as u32);
        unsafe {
            Port::<u32>::new(CFG_ADDR_PORT_NO).write(addr);
            Port::<u32>::new(CFG_DATA_PORT_NO).read()
        }
    }

    /// Check whether this device is invalid.
    /// If invalid, vendor id and device id will set to 0xffff.
    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.read_offset(0x00) == 0xffffffff
    }

    /// read vendor id of this device.
    #[inline]
    pub fn vendor_id(&self) -> u16 {
        // offset 0x00 byte, size 2 bytes
        (self.read_offset(0x00) & 0xffff) as u16
    }

    /// read device id of this device.
    #[inline]
    pub fn device_id(&self) -> u16 {
        // offset 0x02 byte, size 2 bytes
        (self.read_offset(0x00) >> 16) as u16
    }

    /// read class code and revision(1st byte) of this device.
    #[inline]
    pub fn class_code_rev(&self) -> u32 {
        // offset 0x08 byte, size 4 bytes
        self.read_offset(0x08)
    }

    /// read header type of this device.
    #[inline]
    pub fn header_type(&self) -> u8 {
        // offset 0x0e byte, size 1 byte
        ((self.read_offset(0x0c) >> 16) & 0xff) as u8

        // @todo : header type might be used repeatedly. Store in its own field?
    }

    /// whether this device is single-functioned
    #[inline]
    pub fn is_single_fun(&self) -> bool {
        (self.header_type() & 0x80) == 0
    }

    /// read bus num for this device.
    #[inline]
    pub fn bus_num(&self) -> u32 {
        // offset 0x18 byte, size 4 bytes (BAR = Base Address Register)
        self.read_offset(0x18)
    }

}

// fn is_single_fun(header_type: u8) -> bool {
//     (header_type & 0x80) == 0
// }

pub enum Error {
    Full,
    Empty
}
pub type Result = core::result::Result<(), Error>;

const DEVICE_CAP: usize = 32;

/// array of found devices which can hold up to 32 devices.
pub struct Devices {
    store: [Device; DEVICE_CAP],
    count: usize,
}

impl Devices {
    /// get a slice for devices.
    pub fn as_slice(&self) -> &[Device] {
        &self.store[..self.count]
    }

    /// scan devices and create new device store.
    pub fn scan() -> Option<Self> {
        let mut devices = Self { store: [Device::default(); 32], count: 0 };
        if devices.scan_all().is_ok() {
            Some(devices)
        } else { None }
    }

    fn scan_all(&mut self) -> Result {
        // start from bus 0.
        if Device::from_bdf(0, 0, 0).is_single_fun() {
            return self.scan_bus(0);
        }
        for fun in 0..8u8 { // 1..8u8 is said to be buggy
            self.scan_bus(fun)?; // fun as initial bus
        }
        Ok(())
    }

    fn scan_bus(&mut self, bus: u8) -> Result {
        if bus < 8 && Device::from_bdf(0, 0, bus).is_invalid() {
            return Ok(());
        }

        for dev in 0..32u8 {
            self.scan_dev(bus, dev)?;
        }
        Ok(())
    }

    fn scan_dev(&mut self, bus: u8, dev: u8) -> Result {
        let entry_device = Device::from_bdf(bus, dev, 0);
        if entry_device.is_invalid() {
            return Ok(());
        }
        if entry_device.is_single_fun() {
            return self.scan_fun(bus, dev, 0);
        }

        for fun in 0..8u8 {
            self.scan_fun(bus, dev, fun)?;
        }
        Ok(())
    }

    fn scan_fun(&mut self, bus: u8, dev: u8, fun: u8) -> Result {
        let device = Device::from_bdf(bus, dev, fun);
        if device.is_invalid() {
            return Ok(());
        }

        // add dev
        self.add_dev(device)?;

        // check if there are more buses to scan (DFS)
        let class_code_rev = device.class_code_rev();
        let base = ((class_code_rev >> 24) & 0xff) as u8;
        let sub = ((class_code_rev >> 16) & 0xff) as u8;

        if base == 0x06 && sub == 0x04 {
            let bus_num = device.bus_num();
            let next_bus = ((bus_num >> 8) & 0xff) as u8;
            return self.scan_bus(next_bus);
        }
        Ok(())
    }

    fn add_dev(&mut self, device: Device) -> Result {
        if self.count == DEVICE_CAP {
            return Err(Error::Full);
        }
        self.store[self.count] = device;
        self.count += 1;

        Ok(())
    }
}