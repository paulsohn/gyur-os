use x86_64::{instructions::port::Port, PhysAddr};

/// PCI class code (base, sub, interface)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClassCode(u32);

impl ClassCode {
    pub const fn code(&self) -> u32 {
        self.0
    }

    pub const fn base(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    pub const fn sub(&self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    pub const fn interface(&self) -> u8 {
        (self.0 & 0xff) as u8
    }

    pub const fn match_base(&self, base: u8) -> bool {
        self.base() == base
        // self.0 & 0xff0000 == (base as u32) << 16
    }

    pub const fn match_base_sub(&self, base: u8, sub: u8) -> bool {
        self.0 & 0xffff00 == ((base as u32) << 16) | ((sub as u32) << 8)
    }

    pub const fn match_base_sub_interface(&self, base: u8, sub: u8, interface: u8) -> bool {
        self.0 == u32::from_le_bytes([interface, sub, base, 0])
        // self.0 == ((base as u32) << 16) | ((sub as u32) << 8) | (interface as u32)
    }
}

impl From<u32> for ClassCode {
    /// convert 3-bit integer into class code.
    fn from(code: u32) -> Self {
        Self(code & 0xffffff)
    }
}


#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Bar {
    None,
    IO(u32),
    MM(PhysAddr),
}

const CFG_ADDR_PORT_NO: u16 = 0x0cf8;
const CFG_DATA_PORT_NO: u16 = 0x0cfc;

/// PCI device.
/// Internally a device is identified by its config header address.
/// Config info abount this device may be obtained via Port I/O.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Device(u32);

impl Device {
    /// Create config space header address from (bus, slot, fun) triple.
    /// The enable bit is set by default.
    pub const fn from_bsf(bus: u8, slot: u8, fun: u8) -> Self {
        //   1u32         << 31     // enable bit
        // | (bus as u32) << 16     // bus no(8bit)
        // | (slot as u32) << 11    // slot no(5bit)
        // | (fun as u32) <<  8     // fun no(3bit)
        // | (offset & 0xfc) as u32;// offset(8bit), round off last 2bit
        Self(
            // u32::from_le_bytes([0, (slot << 3) | fun, bus, 0x80])
            1u32            << 31     // enable bit
            | (bus as u32)  << 16     // bus no(8bit)
            | (slot as u32) << 11     // slot no(5bit)
            | (fun as u32)  <<  8     // fun no(3bit)
        )
    }

    /// returns `bus` no. of this configuration.
    pub const fn bus(&self) -> u8 {
        // ((self.0 & 0xff0000) >> 16) as u8
        u32::to_le_bytes(self.0)[2]
    }

    /// returns `(slot, fun)` pair of this configuration.
    pub const fn slot_fun(&self) -> (u8, u8) {
        // let slot_fun = ((self.0 & 0xff00) >> 16) as u8;
        let slot_fun = u32::to_le_bytes(self.0)[1];
        ((slot_fun & 0xf8) >> 3, slot_fun & 0x07)
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

    /// read class code of this device.
    #[inline]
    pub fn class_code(&self) -> ClassCode {
        // offset 0x09 byte, size 3 bytes
        ClassCode::from(self.read_offset(0x08) >> 8)
    }

    /// read header type of this device.
    #[inline]
    pub fn header_type(&self) -> u8 {
        // offset 0x0e byte, size 1 byte
        ((self.read_offset(0x0c) >> 16) & 0xff) as u8
    }

    /// whether this device is single-functioned
    #[inline]
    pub fn is_single_fun(&self) -> bool {
        (self.header_type() & 0x80) == 0
    }

    /// read BAR(Base Address Register)
    /// currently this is only intended to read BAR0.
    #[inline]
    pub fn bar0(&self /*, idx: u8 */) -> Bar {
        // BAR bits
        // bit 0: region(0: MM, 1: IO)
        // bit 1~2: bit mode(00: 32bit, 10: 64bit etc.)
        // bit 3: prefetchable(0: no, 1: yes)
        // bit 4~ : actual physical address, 16-byte aligned

        /* let idx = idx.clamp(0, 5); */
        let raw0 = self.read_offset(0x10 /* + 4 * idx */);

        if raw0 & 1 != 0 {
            return Bar::IO(raw0 ^ (raw0 & 0x3));
        }

        let bit_64 = raw0 & 0x6 != 0x6;
        let raw1 = if bit_64 /* && idx < 5 */ {
            self.read_offset(0x10 + 4 /* * (idx + 1) */)
        } else { 0 };
        let raw = ((raw1 as u64) << 32) | (raw0 as u64);

        let addr = raw ^ (raw & 0xf);
        if addr == 0 {
            Bar::None
        } else {
            Bar::MM(PhysAddr::new(addr))
        }
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

/// array of found devices which can hold up to `DEVICE_CAP` devices.
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

    #[allow(dead_code)]
    fn scan_all_brute(&mut self) -> Result {
        for bus in 0..=0xffu8 {
            for slot in 0..=31u8 {
                for fun in 0..=7u8 {
                    let dev = Device::from_bsf(bus, slot, fun);
                    if dev.is_invalid() {
                        continue;
                    }

                    // add dev
                    self.add_dev(dev)?;

                    if fun == 0 && dev.is_single_fun() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn scan_all(&mut self) -> Result {
        // this method uses DFS (with PCI-to-PCI bridges as edges) to scan next devices
        // alternatively, we can just iterate through all 65536 combinations
        // via `self.scan_all_brute()`

        // start from bus 0.
        if Device::from_bsf(0, 0, 0).is_single_fun() {
            return self.scan_bus(0);
        }
        for fun in 0..8u8 { // 1..8u8 is said to be buggy
            // this is the only line accepting "check before visit" policy.
            // on other scenes, DFS is implemented with "check when visit".
            if Device::from_bsf(0, 0, fun).is_invalid() { continue; }
            self.scan_bus(fun)?; // fun as initial bus
        }
        Ok(())
    }

    fn scan_bus(&mut self, bus: u8) -> Result {
        for slot in 0..32u8 {
            self.scan_slot(bus, slot)?;
        }
        Ok(())
    }

    fn scan_slot(&mut self, bus: u8, slot: u8) -> Result {
        let entry_dev = Device::from_bsf(bus, slot, 0);
        if entry_dev.is_invalid() {
            return Ok(());
        }
        if entry_dev.is_single_fun() {
            return self.scan_fun(bus, slot, 0);
        }

        for fun in 0..8u8 {
            self.scan_fun(bus, slot, fun)?;
        }
        Ok(())
    }

    fn scan_fun(&mut self, bus: u8, slot: u8, fun: u8) -> Result {
        let dev = Device::from_bsf(bus, slot, fun);
        if dev.is_invalid() {
            return Ok(());
        }

        // add dev
        self.add_dev(dev)?;

        // scan for PCI-to-PCI bridges
        // if there is any, more buses should be scanned (DFS)
        if dev.class_code().match_base_sub(0x06, 0x04) {
            // `bus_num` reads BAR2.
            // in PCI-to-PCI bridges, BAR2 is used to represent bus_num?

            // let bus_num = dev.read_offset(0x18);
            let secondary_bus = ((dev.read_offset(0x18) >> 8) & 0xff) as u8;
            return self.scan_bus(secondary_bus);
        }
        Ok(())
    }

    fn add_dev(&mut self, dev: Device) -> Result {
        if self.count == DEVICE_CAP {
            return Err(Error::Full);
        }
        self.store[self.count] = dev;
        self.count += 1;

        Ok(())
    }
}