use core::slice;
use vcell::VolatileCell;

pub const PMA_BASE: u32 = 0x4000_6000;
pub const BTABLE_SIZE: u16 = 64;
const BTABLE_ENTRY_SIZE: u16 = 8;

pub fn btable_entry_ptr(ep_id: u8) -> *mut BTableEntry {
    (PMA_BASE + BTABLE_ENTRY_SIZE as u32 * 2 * ep_id as u32) as *mut BTableEntry
}

#[repr(packed)]
pub struct BTableEntry {
    pub tx: BTableTxEntry,
    pub rx: BTableRxEntry,
}
impl BTableEntry {
    #[allow(dead_code)]
    pub unsafe fn from_ep_id(ep_id: u8) -> &'static mut BTableEntry {
        &mut *btable_entry_ptr(ep_id)
    }
}

#[repr(packed)]
pub struct BTableTxEntry {
    addr: VolatileCell<u16>,
    _addr_pad: u16,
    count: VolatileCell<u16>,
    _count_pad: u16,
}
impl BTableTxEntry {
    pub fn addr(&self) -> u16 {
        unsafe { self.addr.get() }
    }
    pub fn set_addr(&mut self, value: u16) {
        unsafe { self.addr.set(value) }
    }

    #[allow(dead_code)]
    pub fn count(&self) -> u16 {
        unsafe { self.count.get() }
    }
    pub fn set_count(&mut self, value: u16) {
        unsafe { self.count.set(value) }
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_slice_mut(&self, len: usize) -> &mut [VolatileCell<u16>] {
        let addr = self.addr() as u32 * 2 + PMA_BASE;
        slice::from_raw_parts_mut(addr as *mut VolatileCell<u16>, len)
    }
}

#[repr(packed)]
pub struct BTableRxEntry {
    addr: VolatileCell<u16>,
    _addr_pad: u16,
    count: VolatileCell<u16>,
    _count_pad: u16,
}
impl BTableRxEntry {
    pub fn addr(&self) -> u16 {
        unsafe { self.addr.get() }
    }
    pub fn set_addr(&mut self, value: u16) {
        unsafe { self.addr.set(value) }
    }

    pub fn count(&self) -> u16 {
        unsafe { self.count.get() }
    }
    pub fn set_count(&mut self, value: u16) {
        unsafe { self.count.set(value) }
    }

    pub fn set_buf_size(&mut self, min_size: u16) -> u16 {
        if min_size > 62 {
            let num_block = ((min_size - 1) >> 5) & 0b1_1111;
            self.set_count((num_block | 0b10_0000) << 10);
            (num_block + 1) << 5
        } else {
            let num_block = (min_size + 1) >> 1;
            self.set_count(num_block << 10);
            num_block << 1
        }
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        let addr = self.addr() as u32 * 2 + PMA_BASE;
        slice::from_raw_parts(addr as *mut u8, self.count() as usize)
    }
}

pub fn copy_to_pm(pm: &[VolatileCell<u16>], buf: &[u8]) {
    let pm_iter = pm.iter().step_by(2);
    let buf16 = unsafe { slice::from_raw_parts((buf as *const [u8]) as *const u16, buf.len() >> 1) };
    let buf_iter = buf16.iter();
    for (dst, src) in pm_iter.zip(buf_iter) {
        dst.set(*src);
    }
    if buf.len() & 1 == 1 {
        pm.last().unwrap().set(*buf.last().unwrap() as u16);
    }
}

pub fn copy_from_pm(buf: &mut [u8], pm: &[u8]) {
    let pm_iter = pm.chunks(2).step_by(2).flatten();
    let buf_iter_mut = buf.iter_mut();
    for (dst, src) in buf_iter_mut.zip(pm_iter) {
        *dst = *src;
    }
}

pub unsafe fn fill_with_zero() {
    let pm = slice::from_raw_parts_mut(PMA_BASE as *mut u8, 512);
    for byte in pm.iter_mut() {
        *byte = 0;
    }
}
