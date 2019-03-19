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
            .iopcen().set_bit()
    });
    p.GPIOC.crh.write(|w| unsafe { w
        .mode13().bits(gpio::Mode::Output2MHz.bits())
        .cnf13().bits(gpio::OutputCnf::Opendrain.bits())
    });

    loop {
        p.GPIOC.odr.write(|w| {
            w.odr13().bit(true)
        });
        for _ in 0..80000 {
            cortex_m::asm::nop();
        }
        p.GPIOC.odr.write(|w| {
            w.odr13().bit(false)
        });
        for _ in 0..80000 {
            cortex_m::asm::nop();
        }
    }
}
