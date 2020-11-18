#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f7xx_hal::i2c::{BlockingI2c, I2c, Mode};
use stm32f7xx_hal::pac;
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::rcc::{HSEClock, HSEClockMode};

const ADDR: u8 = 0x34 >> 1;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .hse(HSEClock::new(25.mhz(), HSEClockMode::Bypass))
        .use_pll()
        .use_pll48clk()
        .sysclk(216.mhz())
        .freeze();

    let gpiob = dp.GPIOB.split();
    let scl = gpiob.pb8.into_alternate_af4().set_open_drain();
    let sda = gpiob.pb9.into_alternate_af4().set_open_drain();

    let mut i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        Mode::Standard {
            frequency: 400_000.hz(),
        },
        clocks,
        &mut rcc.apb1,
        10000,
    );

    cortex_m::asm::delay(216_000_000); 

    loop {        
        cortex_m::asm::delay(216_000_000); 
        let mut buf = [0u8; 2];
        let res = i2c.write_read(ADDR, &0x00u32.to_be_bytes(), &mut buf);
        rprintln!("Device ID: {:x}", u16::from_be_bytes(buf));
        
        cortex_m::asm::delay(216_000_000); 
        let mut buf = [0u8; 2];
        i2c.write_read(ADDR, &(0x1Cu16).to_be_bytes(), &mut buf).ok();
        rprintln!("Vol: {:x}", u16::from_be_bytes(buf));
        
        let mut buf = [0u8; 4];
        const REG: u16 = 0x1C;
        let data = 0x11u16;
        buf[..2].copy_from_slice(&REG.to_be_bytes());
        buf[2..].copy_from_slice(&data.to_be_bytes());
        i2c.write(ADDR, &buf).ok();

        cortex_m::asm::delay(216_000_000); 
        let mut buf = [0u8; 2];
        i2c.write_read(ADDR, &(0x1Cu16).to_be_bytes(), &mut buf).ok();
        rprintln!("Vol: {:x}", u16::from_be_bytes(buf));
        
    }
}
