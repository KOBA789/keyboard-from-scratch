#![no_std]
#![no_main]

use panic_halt as _;

use core::cmp;

use cortex_m_rt::entry;
use stm32f1::stm32f103;
use stm32f103::USB as USBRegs;
use vcell::VolatileCell;

#[allow(unused_imports)]
use cortex_m_semihosting::hprintln;

mod cursor;
mod descr;
mod gpio;
mod pma;

use cursor::{ReadCursor, WriteCursor};

static DEVICE_DESCR: descr::DeviceDescriptor = descr::DeviceDescriptor {
    bLength: core::mem::size_of::<descr::DeviceDescriptor>() as u8,
    bDescriptorType: 1,
    bcdUSB: 0x0200,
    bDeviceClass: 0,
    bDeviceSubClass: 0,
    bDeviceProtocol: 0,
    bMaxPacketSize0: 64,
    idVendor: 0x0483,
    idProduct: 0x5710,
    bcdDevice: 0x0200,
    iManufacturer: 1,
    iProduct: 2,
    iSerialNumber: 3,
    bNumConfigurations: 1,
};

#[repr(packed)]
pub struct CompositeConfigDescriptor {
    pub config: descr::ConfigDescriptor,
    pub kbd_interf: descr::InterfaceDescriptor,
    pub hid_func: descr::HidFunction,
    pub hid_endpoint: descr::EndpointDescriptor,
}

static CONFIG_DESCR: CompositeConfigDescriptor = CompositeConfigDescriptor {
    config: descr::ConfigDescriptor {
        bLength: core::mem::size_of::<descr::ConfigDescriptor>() as u8,
        bDescriptorType: 2,
        wTotalLength: core::mem::size_of::<CompositeConfigDescriptor>() as u16,
        bNumInterfaces: 1,
        bConfigurationValue: 1,
        iConfiguration: 0,
        bmAttributes: 0xC0,
        bMaxPower: 0x32,
    },
    kbd_interf: descr::InterfaceDescriptor {
        bLength: core::mem::size_of::<descr::InterfaceDescriptor>() as u8,
        bDescriptorType: 4,
        bInterfaceNumber: 0,
        bAlternateSetting: 0,
        bNumEndpoints: 1,
        bInterfaceClass: 3, // = USB_CLASS_HID
        bInterfaceSubClass: 1,
        bInterfaceProtocol: 1,
        iInterface: 0,
    },
    hid_func: descr::HidFunction {
        hid_descriptor: descr::HidDescriptor {
            bLength: core::mem::size_of::<descr::HidFunction>() as u8,
            bDescriptorType: 0x21,
            bcdHID: 0x0101,
            bCountryCode: 0,
            bNumDescriptors: 1,
        },
        hid_report: descr::HidReport {
            bReportDescriptorType: 0x22,
            wDescriptorLength: 63,
        },
    },
    hid_endpoint: descr::EndpointDescriptor {
        bLength: core::mem::size_of::<descr::EndpointDescriptor>() as u8,
        bDescriptorType: 5,
        bEndpointAddress: 0x81,
        bmAttributes: 0x03,
        wMaxPacketSize: 8,
        bInterval: 0x0a,
    },
};

static HID_REPORT_DESCR: &[u8] = &[
    0x05, 0x01, 0x09, 0x06, 0xA1, 0x01, 0x05, 0x07, 0x19, 0xE0, 0x29, 0xE7, 0x15, 0x00, 0x25, 0x01,
    0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x95, 0x01, 0x75, 0x08, 0x81, 0x01, 0x95, 0x06, 0x75, 0x01,
    0x05, 0x08, 0x19, 0x01, 0x29, 0x05, 0x91, 0x02, 0x95, 0x01, 0x75, 0x03, 0x91, 0x01, 0x95, 0x06,
    0x75, 0x08, 0x15, 0x00, 0x25, 0x65, 0x05, 0x07, 0x19, 0x00, 0x29, 0x65, 0x81, 0x00, 0xC0,
];

static STRINGS: &[&str] = &["KOBA789", "KB789 MK-C", "789"];

fn setup_clock(rcc: &stm32f103::RCC, flash: &stm32f103::FLASH) {
    rcc.cr.write(|w| w.hsion().set_bit());
    while rcc.cr.read().hsirdy().bit_is_clear() {}
    rcc.cfgr.write(|w| w.sw().hsi());

    rcc.cr.write(|w| w.hsion().set_bit().hseon().set_bit());
    while rcc.cr.read().hserdy().bit_is_clear() {}
    rcc.cfgr.write(|w| w.sw().hse());

    flash.acr.write(|w| unsafe { w.latency().bits(0b010) });

    rcc.cfgr.write(|w| {
        w.sw()
            .hse()
            .hpre()
            .div1()
            .adcpre()
            .div8()
            .ppre1()
            .div2()
            .ppre2()
            .div1()
            .pllmul()
            .mul9()
            .pllsrc()
            .hse_div_prediv()
            .pllxtpre()
            .div1()
    });

    rcc.cr
        .write(|w| w.hsion().set_bit().hseon().set_bit().pllon().set_bit());
    while rcc.cr.read().pllrdy().bit_is_clear() {}
    rcc.cfgr.write(|w| w.sw().pll());
}

#[derive(Debug, PartialEq)]
enum Direction {
    HostToDevice,
    DeviceToHost,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum Type {
    Standard,
    Class,
    Vendor,
    Reserved,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum Recipient {
    Device,
    Interface,
    Endpoint,
    Other,
    Reserved,
}

#[repr(packed)]
#[derive(Debug)]
pub struct BmRequestType(u8);
impl BmRequestType {
    #[inline]
    fn bits(&self) -> u8 {
        self.0
    }

    #[inline]
    fn direction(&self) -> Direction {
        if self.bits() & 0x80 == 0 {
            Direction::HostToDevice
        } else {
            Direction::DeviceToHost
        }
    }

    #[inline]
    #[allow(dead_code)]
    fn request_type(&self) -> Type {
        match (self.bits() >> 5) & 0b11 {
            0 => Type::Standard,
            1 => Type::Class,
            2 => Type::Vendor,
            3 => Type::Reserved,
            _ => unreachable!(),
        }
    }

    #[inline]
    #[allow(dead_code)]
    fn recipient(&self) -> Recipient {
        match self.bits() & 0b11111 {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            _ => Recipient::Reserved,
        }
    }
}

#[repr(packed)]
#[allow(non_snake_case)]
pub struct DeviceRequest {
    pub bmRequestType: BmRequestType,
    pub bRequest: u8,
    pub wValue: u16,
    pub wIndex: u16,
    pub wLength: u16,
}

#[repr(packed)]
#[derive(Clone, Copy)]
struct EPAddr(u8);
impl EPAddr {
    fn new(bits: u8) -> Self {
        EPAddr(bits)
    }

    #[allow(dead_code)]
    fn from(dir: Direction, ep_id: u8) -> Self {
        match dir {
            Direction::DeviceToHost => Self::new(ep_id | 0x80),
            Direction::HostToDevice => Self::new(ep_id),
        }
    }

    fn bits(&self) -> u8 {
        self.0
    }

    fn dir(&self) -> Direction {
        if self.bits() & 0x80 == 0 {
            Direction::HostToDevice
        } else {
            Direction::DeviceToHost
        }
    }

    fn ep_id(&self) -> u8 {
        self.bits() & 0x7f
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum EPType {
    Bulk,
    Control,
    Isochronous,
    Interrupt,
}
impl EPType {
    fn bits(&self) -> u8 {
        use EPType::*;
        match self {
            Bulk => 0b00,
            Control => 0b01,
            Isochronous => 0b10,
            Interrupt => 0b11,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum EPStat {
    Disabled,
    Stall,
    Nak,
    Valid,
}
impl EPStat {
    fn bits(&self) -> u8 {
        use EPStat::*;
        match self {
            Disabled => 0b00,
            Stall => 0b01,
            Nak => 0b10,
            Valid => 0b11,
        }
    }
}

mod ep {
    use stm32f1::stm32f103::usb::epr::{R as EPR_R, W as EPR_W};

    pub fn invariant(w: &mut EPR_W) -> &mut EPR_W {
        w.ctr_rx()
            .set_bit()
            .dtog_rx()
            .clear_bit()
            .stat_rx()
            .bits(0)
            .ctr_tx()
            .set_bit()
            .dtog_tx()
            .clear_bit()
            .stat_tx()
            .bits(0)
    }

    pub fn clear_tx_dtog<'w>(r: &EPR_R, w: &'w mut EPR_W) -> &'w mut EPR_W {
        w.dtog_tx().bit(r.dtog_tx().bit())
    }
    pub fn clear_rx_dtog<'w>(r: &EPR_R, w: &'w mut EPR_W) -> &'w mut EPR_W {
        w.dtog_tx().bit(r.dtog_tx().bit())
    }

    pub fn set_tx_stat<'w>(r: &EPR_R, w: &'w mut EPR_W, stat: u8) -> &'w mut EPR_W {
        w.stat_tx().bits(r.stat_tx().bits() ^ stat)
    }
    pub fn set_rx_stat<'w>(r: &EPR_R, w: &'w mut EPR_W, stat: u8) -> &'w mut EPR_W {
        w.stat_rx().bits(r.stat_rx().bits() ^ stat)
    }
}

#[allow(dead_code)]
enum ControlState<'a> {
    Idle {
        buf: &'a mut [u8],
    },
    Stalled {
        buf: &'a mut [u8],
    },
    DataIn {
        cur: ReadCursor<'a>,
        req: DeviceRequest,
    },
    LastDataIn {
        cur: ReadCursor<'a>,
        req: DeviceRequest,
    },
    StatusIn {
        buf: &'a mut [u8],
    },
    DataOut {
        cur: WriteCursor<'a>,
        req: DeviceRequest,
    },
    LastDataOut {
        cur: WriteCursor<'a>,
        req: DeviceRequest,
    },
    StatusOut {
        buf: &'a mut [u8],
    },
}
impl<'a> ControlState<'a> {
    fn into_buf(self) -> &'a mut [u8] {
        use ControlState::*;
        match self {
            Idle { buf } => buf,
            Stalled { buf } => buf,
            DataIn { cur, .. } => cur.into(),
            LastDataIn { cur, .. } => cur.into(),
            StatusIn { buf } => buf,
            DataOut { cur, .. } => cur.into(),
            LastDataOut { cur, .. } => cur.into(),
            StatusOut { buf } => buf,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RequestStatus {
    NotSupported,
    Handled,
}

struct USBKbd<'a> {
    regs: USBRegs,
    device_descr: &'a descr::DeviceDescriptor,
    config_descr: &'a [u8],
    ctrl_state: ControlState<'a>,
    pending_addr: Option<u8>,
    pm_top: u16,
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of::<T>())
}

impl<'a> USBKbd<'a> {
    fn new(
        regs: USBRegs,
        device_descr: &'static descr::DeviceDescriptor,
        config_descr: &'static [u8],
        ctrl_buf: &'a mut [u8],
    ) -> Self {
        USBKbd {
            regs,
            device_descr,
            config_descr,
            ctrl_state: ControlState::Idle { buf: ctrl_buf },
            pending_addr: None,
            pm_top: pma::BTABLE_SIZE,
        }
    }

    fn setup(&mut self) {
        self.reset();
        self.regs
            .cntr
            .write(|w| w.pdwn().clear_bit().resetm().set_bit().ctrm().set_bit());
        self.regs.cntr.modify(|_, w| w.fres().clear_bit());
    }

    fn set_addr(&self, addr: u8) {
        self.regs
            .daddr
            .modify(|_, w| w.add().bits(addr).ef().set_bit());
    }

    fn reset(&mut self) {
        self.regs.istr.reset();
        self.pm_top = pma::BTABLE_SIZE;
        self.ep_setup(
            EPAddr::new(0),
            EPType::Control,
            self.device_descr.bMaxPacketSize0 as u16,
        );
        self.set_addr(0);
    }

    fn epr(&self, ep_id: u8) -> &stm32f103::usb::EPR {
        &self.regs.epr[ep_id as usize]
    }

    fn ep_bt_entry(&mut self, addr: EPAddr) -> &mut pma::BTableEntry {
        let ptr = pma::btable_entry_ptr(addr.ep_id());
        unsafe { &mut *ptr }
    }

    fn ep_pm_tx(&mut self, addr: EPAddr, len: usize) -> &mut [VolatileCell<u16>] {
        let entry = self.ep_bt_entry(addr);
        entry.tx.set_count(len as u16);
        unsafe { entry.tx.as_slice_mut(len) }
    }

    fn ep_pm_rx(&mut self, addr: EPAddr) -> &[u8] {
        unsafe { self.ep_bt_entry(addr).rx.as_slice() }
    }

    fn ep_clear_ctr_tx(&mut self, ep_id: u8) {
        self.epr(ep_id)
            .modify(|_, w| ep::invariant(w).ctr_tx().clear_bit());
    }

    fn ep_clear_ctr_rx(&mut self, ep_id: u8) {
        self.epr(ep_id)
            .modify(|_, w| ep::invariant(w).ctr_rx().clear_bit());
    }

    fn ep_stall(&mut self, addr: EPAddr) {
        self.epr(addr.ep_id()).modify(|r, w| {
            let w = ep::invariant(w);
            let w = if addr.ep_id() == 0 {
                ep::set_tx_stat(r, w, EPStat::Stall.bits())
            } else {
                w
            };
            match addr.dir() {
                Direction::HostToDevice => ep::set_rx_stat(r, w, EPStat::Stall.bits()),
                Direction::DeviceToHost => ep::set_tx_stat(r, w, EPStat::Stall.bits()),
            }
        });
    }

    fn ep_setup(&mut self, addr: EPAddr, ep_type: EPType, size: u16) {
        {
            let epr = self.epr(addr.ep_id());
            epr.modify(|_, w| {
                let w = ep::invariant(w);
                w.ea().bits(addr.bits()).ep_type().bits(ep_type.bits())
            });
        }

        // IN or control ep
        if addr.dir() == Direction::DeviceToHost || ep_type == EPType::Control {
            let tx_addr = self.pm_top;
            let entry = self.ep_bt_entry(addr);
            entry.tx.set_addr(tx_addr);
            entry.tx.set_count(0);
            let epr = self.epr(addr.ep_id());
            epr.modify(|r, w| {
                let w = ep::invariant(w);
                let w = ep::clear_tx_dtog(r, w);
                ep::set_tx_stat(r, w, EPStat::Nak.bits())
            });
            self.pm_top += size;
        }
        // OUT
        if addr.dir() == Direction::HostToDevice {
            let rx_addr = self.pm_top;
            let entry = self.ep_bt_entry(addr);
            entry.rx.set_addr(rx_addr);
            let realsize = entry.rx.set_buf_size(size);
            let epr = self.epr(addr.ep_id());
            epr.modify(|r, w| {
                let w = ep::invariant(w);
                let w = ep::clear_rx_dtog(r, w);
                ep::set_rx_stat(r, w, EPStat::Valid.bits())
            });
            self.pm_top += realsize;
        }
    }

    fn ep_write_packet(&mut self, addr: EPAddr, buf: &[u8]) -> Option<()> {
        if self.epr(addr.ep_id()).read().stat_tx().bits() == EPStat::Valid.bits() {
            return None;
        }
        let pm = self.ep_pm_tx(addr, buf.len());
        pma::copy_to_pm(pm, buf);
        self.epr(addr.ep_id()).modify(|r, w| {
            let w = ep::invariant(w);
            ep::set_tx_stat(r, w, EPStat::Valid.bits())
        });
        Some(())
    }

    fn ep_read_packet(&mut self, addr: EPAddr, buf: &mut [u8]) -> Option<()> {
        if self.epr(addr.ep_id()).read().stat_rx().bits() == EPStat::Valid.bits() {
            return None;
        }

        let pm = self.ep_pm_rx(addr);
        pma::copy_from_pm(buf, pm);
        self.ep_clear_ctr_rx(addr.ep_id());
        self.epr(addr.ep_id()).modify(|r, w| {
            let w = ep::invariant(w);
            ep::set_rx_stat(r, w, EPStat::Valid.bits())
        });
        Some(())
    }

    fn ctrl_transition<F>(&mut self, cb: F)
    where
        F: FnOnce(&mut Self, ControlState<'a>) -> ControlState<'a>,
    {
        let state = core::mem::replace(&mut self.ctrl_state, ControlState::Idle { buf: &mut [] });
        self.ctrl_state = cb(self, state);
    }

    fn ctrl_handle_out(&mut self) {
        if self.regs.epr[0].read().setup().bit() {
            self.ctrl_handle_setup();
            return;
        }

        use ControlState::*;
        self.ctrl_transition(|this, state| {
            match state {
                /*
                DataOut { cur, req } => {
                    this.ctrl_send_chunk(cur, req, needs_zlp)
                },
                */
                StatusOut { buf, .. } => {
                    this.ep_read_packet(EPAddr::new(0), &mut []);
                    ControlState::Idle { buf }
                }
                _ => {
                    this.ep_stall(EPAddr::new(0));
                    ControlState::Stalled {
                        buf: state.into_buf(),
                    }
                }
            }
        });
    }

    fn ctrl_read_req(&mut self) -> DeviceRequest {
        let mut buf = [0u8; core::mem::size_of::<DeviceRequest>()];
        self.ep_read_packet(EPAddr::new(0), &mut buf).unwrap();
        unsafe { core::mem::transmute(buf) }
    }

    fn ctrl_handle_setup(&mut self) {
        let req = self.ctrl_read_req();
        if req.wLength == 0 {
            self.ctrl_setup_read(req);
        } else {
            match req.bmRequestType.direction() {
                Direction::HostToDevice => self.ctrl_setup_write(req),
                Direction::DeviceToHost => self.ctrl_setup_read(req),
            }
        }
    }

    fn ctrl_setup_read(&mut self, req: DeviceRequest) {
        self.ctrl_transition(|this, state| {
            let buf = state.into_buf();
            let mut wcur = WriteCursor::new(buf);
            match this.ctrl_handle_read_request(&req, &mut wcur) {
                RequestStatus::NotSupported => {
                    this.ep_stall(EPAddr::new(0));
                    ControlState::Stalled {
                        buf: wcur.into_buf(),
                    }
                }
                RequestStatus::Handled => {
                    if req.wLength == 0 {
                        this.ep_write_packet(EPAddr::new(0), &[]).unwrap();
                        ControlState::StatusIn {
                            buf: wcur.into_buf(),
                        }
                    } else {
                        let cur = wcur.into_read();
                        this.ctrl_send_chunk(cur, req)
                    }
                }
            }
        });
    }

    fn ctrl_setup_write(&mut self, req: DeviceRequest) {
        self.ctrl_handle_write_request(req, &[]);
    }

    fn ctrl_handle_read_request(
        &mut self,
        req: &DeviceRequest,
        wcur: &mut WriteCursor<'a>,
    ) -> RequestStatus {
        let mut str_buf = [0u8; 64];
        match req.bRequest {
            0x05 => {
                // SET_ADDRESS
                self.pending_addr = Some(req.wValue as u8);
                //self.set_addr(req.wValue as u8);
                //hprintln!("{:x?}", req.wValue);
                RequestStatus::Handled
            }
            0x06 => {
                // GET_DESCRIPTOR
                let descr_index = req.wValue & 0xff;
                let bytes = match req.wValue & 0xff00 {
                    0x0100 => unsafe { any_as_u8_slice(&*self.device_descr) },
                    0x0200 => self.config_descr,
                    0x0300 => {
                        if descr_index == 0 {
                            descr::STRING_DESCR0
                        } else {
                            let str_data = STRINGS[descr_index as usize - 1];
                            let len = descr::build_string_descr(&mut str_buf, str_data).unwrap();
                            &str_buf[0..len]
                        }
                    }
                    0x2200 => HID_REPORT_DESCR,
                    _ => {
                        return RequestStatus::NotSupported;
                    }
                };
                let len = cmp::min(req.wLength as usize, bytes.len());
                wcur.write(&bytes[0..len]);
                RequestStatus::Handled
            }
            0x09 => {
                // SET_CONFIGURATION
                self.ep_setup(EPAddr::new(0x81), EPType::Interrupt, 8);
                RequestStatus::Handled
            }
            _ => RequestStatus::NotSupported,
        }
    }

    fn ctrl_handle_write_request(&mut self, _req: DeviceRequest, _buf: &[u8]) {}

    fn ctrl_handle_in(&mut self) {
        use ControlState::*;
        self.ctrl_transition(|this, state| match state {
            DataIn { cur, req } => this.ctrl_send_chunk(cur, req),
            LastDataIn { cur, .. } => {
                this.epr(0).modify(|r, w| {
                    let w = ep::invariant(w);
                    ep::set_rx_stat(r, w, EPStat::Valid.bits())
                });
                this.ep_read_packet(EPAddr::new(0), &mut []);
                let buf = cur.into_buf();
                ControlState::StatusOut { buf }
            }
            StatusIn { buf, .. } => {
                if let Some(addr) = this.pending_addr {
                    this.set_addr(addr);
                    this.pending_addr = None;
                }
                ControlState::Idle { buf }
            }
            _ => {
                this.ep_stall(EPAddr::new(0));
                ControlState::Stalled {
                    buf: state.into_buf(),
                }
            }
        });
    }

    fn ctrl_send_chunk(&mut self, mut cur: ReadCursor<'a>, req: DeviceRequest) -> ControlState<'a> {
        #[allow(non_snake_case)]
        let bMaxPacketSize0 = self.device_descr.bMaxPacketSize0 as usize;

        let chunk = cur.read(bMaxPacketSize0);
        self.ep_write_packet(EPAddr::new(0), chunk).unwrap();

        if bMaxPacketSize0 > chunk.len() {
            ControlState::LastDataIn { cur, req }
        } else {
            ControlState::DataIn { cur, req }
        }
    }

    fn hid_handle_in(&mut self) {}

    fn hid_send_keys(&mut self, keys: &[u8]) -> Option<()> {
        self.ep_write_packet(EPAddr::new(0x81), keys)
    }

    fn usb_poll(&mut self) {
        let istr_r = self.regs.istr.read();
        if istr_r.reset().bit() {
            self.reset();
            return;
        }

        let ep_id = istr_r.ep_id().bits();
        if istr_r.ctr().bit() {
            if istr_r.dir().bit() {
                // OUT
                match ep_id {
                    0 => self.ctrl_handle_out(),
                    _ => self.ep_clear_ctr_rx(ep_id),
                }
            } else {
                // IN
                self.ep_clear_ctr_tx(ep_id);
                match ep_id {
                    0 => self.ctrl_handle_in(),
                    1 => self.hid_handle_in(),
                    _ => {}
                }
            }
        }
    }
}

#[entry]
fn main() -> ! {
    let p = stm32f103::Peripherals::take().unwrap();
    setup_clock(&p.RCC, &p.FLASH);

    p.RCC
        .apb2enr
        .write(|w| w.iopaen().set_bit().iopben().set_bit().iopcen().set_bit());
    p.RCC.apb1enr.write(|w| w.usben().set_bit());
    p.GPIOA.crh.write(|w| {
        w.mode8()
            .bits(gpio::Mode::Output2MHz.bits())
            .cnf8()
            .bits(gpio::OutputCnf::Pushpull.bits())
            .mode12()
            .bits(gpio::Mode::Output50MHz.bits())
            .cnf12()
            .bits(gpio::OutputCnf::Pushpull.bits())
    });
    p.GPIOB.crl.write(|w| {
        w.mode5()
            .bits(gpio::Mode::Input.bits())
            .cnf5()
            .bits(gpio::InputCnf::PullUpdown.bits())
            .mode6()
            .bits(gpio::Mode::Input.bits())
            .cnf6()
            .bits(gpio::InputCnf::PullUpdown.bits())
            .mode7()
            .bits(gpio::Mode::Input.bits())
            .cnf7()
            .bits(gpio::InputCnf::PullUpdown.bits())
    });
    p.GPIOB.crh.write(|w| {
        w.mode11()
            .bits(gpio::Mode::Output2MHz.bits())
            .cnf11()
            .bits(gpio::OutputCnf::Pushpull.bits())
    });
    p.GPIOC.crh.write(|w| {
        w.mode13()
            .bits(gpio::Mode::Output2MHz.bits())
            .cnf13()
            .bits(gpio::OutputCnf::Opendrain.bits())
    });

    p.GPIOA.odr.write(|w| w.odr12().clear_bit());
    for _ in 0..80000 {
        cortex_m::asm::nop();
    }

    unsafe {
        pma::fill_with_zero();
    }
    let mut ctrl_buf = [0u8; 128];
    let config_descr_buf = unsafe {
        core::slice::from_raw_parts(
            (&CONFIG_DESCR as *const CompositeConfigDescriptor) as *const u8,
            core::mem::size_of::<CompositeConfigDescriptor>(),
        )
    };
    let mut kbd = USBKbd::new(p.USB, &DEVICE_DESCR, config_descr_buf, &mut ctrl_buf);
    kbd.setup();
    p.GPIOA.odr.write(|w| w.odr8().bit(true));
    loop {
        kbd.usb_poll();

        let mut buf = [0u8; 8];
        let bit = p.GPIOB.idr.read().idr5().bit();
        if bit {
            buf[2] = 0x04;
        }
        kbd.hid_send_keys(&buf);
    }
}
