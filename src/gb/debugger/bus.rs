use crate::gb::AddressSpace;
use crate::gb::bus::Bus;

pub struct DebugBus {
    inner: Bus,
}

impl From<Bus> for DebugBus {
    #[inline]
    fn from(bus: Bus) -> Self {
        Self { inner: bus }
    }
}

impl AddressSpace for DebugBus {
    #[inline]
    fn write(&mut self, address: u16, value: u8) {
        self.inner.write_raw(address, value);
    }

    #[inline]
    fn read(&mut self, address: u16) -> u8 {
        self.inner.read_raw(address)
    }
}
