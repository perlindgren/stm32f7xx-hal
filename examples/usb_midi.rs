//! CDC-ACM serial port example using polling in a busy loop.
//! Target board: any STM32F7 with a OTG FS peripheral and a 25MHz HSE generator
#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
#[cfg(feature = "usb_fs")]
use stm32f7xx_hal::otg_fs::{UsbBus, USB};
#[cfg(feature = "usb_hs")]
use stm32f7xx_hal::otg_hs::{UsbBus, USB};
use stm32f7xx_hal::pac;
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::rcc::{HSEClock, HSEClockMode};
use usb_device::prelude::*;
use usbd_midi::{
    data::{
        byte::{from_traits::FromClamped, u7::U7},
        midi::{channel::Channel, message::message::Message::NoteOn, notes::Note},
        usb_midi::{cable_number::CableNumber, usb_midi_event_packet::UsbMidiEventPacket},
    },
    midi_device::MidiClass,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("usb midi example");

    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .hse(HSEClock::new(25.mhz(), HSEClockMode::Bypass))
        .use_pll()
        .use_pll48clk()
        .sysclk(216.mhz())
        .freeze();

    #[cfg(feature = "usb_fs")]
    let gpioa = dp.GPIOA.split();
    #[cfg(feature = "usb_hs")]
    let gpiob = dp.GPIOB.split();

    #[cfg(feature = "usb_fs")]
    let usb = USB::new(
        dp.OTG_FS_GLOBAL,
        dp.OTG_FS_DEVICE,
        dp.OTG_FS_PWRCLK,
        (
            gpioa.pa11.into_alternate_af10(),
            gpioa.pa12.into_alternate_af10(),
        ),
        clocks,
    );
    #[cfg(feature = "usb_hs")]
    let usb = USB::new(
        dp.OTG_HS_GLOBAL,
        dp.OTG_HS_DEVICE,
        dp.OTG_HS_PWRCLK,
        (
            gpiob.pb14.into_alternate_af12(),
            gpiob.pb15.into_alternate_af12(),
        ),
        clocks,
    );

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];
    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    let mut midi = MidiClass::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("per.lindgren")
        .product("midi device")
        .serial_number("TEST")
        .device_class(USB_CLASS_NONE)
        .build();

    let mut t = 0;

    loop {
        if !usb_dev.poll(&mut [&mut midi]) {
            continue;
        }
        if usb_dev.state() == UsbDeviceState::Configured {
            rprintln!("configured");
            t += 1;
            //if t == 10000 {
            rprintln!("send note on");
            t = 0;
            let _ = midi.send_message(UsbMidiEventPacket::from_midi(
                CableNumber::Cable0,
                NoteOn(Channel::Channel1, Note::C2, U7::from_clamped(64)),
            ));
        //}
        } else {
            rprintln!("un configured");
        }
    }
}

pub const USB_CLASS_NONE: u8 = 0x00;

// let mut buf = [0u8; 64];

// match serial.read(&mut buf) {
//     Ok(count) if count > 0 => {
//         // Echo back in upper case
//         for c in buf[0..count].iter_mut() {
//             if 0x61 <= *c && *c <= 0x7a {
//                 *c &= !0x20;
//             }
//         }

//         let mut write_offset = 0;
//         while write_offset < count {
//             match serial.write(&buf[write_offset..count]) {
//                 Ok(len) if len > 0 => {
//                     write_offset += len;
//                 }
//                 _ => {}
//             }
//         }
//     }
//     _ => {}
// }
