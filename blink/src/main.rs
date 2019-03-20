#![no_std]
#![no_main]

extern crate panic_halt;

use core::cmp;

use cortex_m;
use cortex_m_rt::entry;
use stm32f1::stm32f103;

mod gpio;

#[entry]
fn main() -> ! {
    let p = stm32f103::Peripherals::take().unwrap();
    p.RCC.apb2enr.write(|w| {
        w
            .iopaen().set_bit()
            .iopben().set_bit()
            .iopcen().set_bit()
    });
    p.GPIOA.crh.write(|w| unsafe { w
        .mode8().bits(gpio::Mode::Output2MHz.bits())
        .cnf8().bits(gpio::OutputCnf::Pushpull.bits())
    });
    p.GPIOB.crl.write(|w| unsafe { w
        .mode5().bits(gpio::Mode::Input.bits())
        .cnf5().bits(gpio::InputCnf::PullUpdown.bits())
    });
    p.GPIOC.crh.write(|w| unsafe { w
        .mode13().bits(gpio::Mode::Output2MHz.bits())
        .cnf13().bits(gpio::OutputCnf::Opendrain.bits())
    });

    p.GPIOA.odr.write(|w| {
        w.odr8().bit(true)
    });

    loop {
        let bit = p.GPIOB.idr.read().idr5().bit();
        p.GPIOC.odr.write(|w| {
            w.odr13().bit(bit)
        });
    }
}
