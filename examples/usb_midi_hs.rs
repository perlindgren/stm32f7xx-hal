//! MIDI out from target.
//! Target board: any STM32F7 with a OTG FS peripheral and a 25MHz HSE generator
//! Tested on STM32F723 Discovery to work with both LINUX and OSX.
//! The application simply outputs midi note on/off messages at an interval of approx 1s.
//!
//! > cargo run --example usb_midi --features  "stm32f723, rt, usb_fs" --release
//!
//! Under linux, in another terminal
//!
//! > aseqdump -p 28
//! waiting for data. Press Ctrl+C to end.
//! Source  Event                  Ch  Data
//! 28:0   Note on                 0, note 36, velocity 64
//! 28:0   Note off                0, note 36, velocity 64

#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;

use stm32f7xx_hal::otg_hs::{UsbBus, USB};
use stm32f7xx_hal::pac;
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::rcc::{HSEClock, HSEClockMode};
use usb_device::prelude::*;
use usbd_midi::{
    data::{
        byte::{from_traits::FromClamped, u7::U7},
        midi::{
            channel::Channel,
            message::message::Message::{NoteOff, NoteOn},
            notes::Note,
        },
        usb_midi::{cable_number::CableNumber, usb_midi_event_packet::UsbMidiEventPacket},
    },
    midi_device::MidiClass,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("usb midi HS example");

    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .hse(HSEClock::new(25.mhz(), HSEClockMode::Bypass))
        .use_pll()
        .use_pll48clk()
        .sysclk(216.mhz())
        .freeze();

    let gpiob = dp.GPIOB.split();

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
        .max_packet_size_0(64)
        .build();

    loop {
        rprintln!("poll");
        if !usb_dev.poll(&mut [&mut midi]) {
            continue;
        }
        rprintln!("after poll");

        let mut state = false;
        while usb_dev.state() == UsbDeviceState::Configured {
            rprintln!("configured");

            if state {
                rprintln!("send note off");
                let _ = midi.send_message(UsbMidiEventPacket::from_midi(
                    CableNumber::Cable0,
                    NoteOff(Channel::Channel1, Note::C2, U7::from_clamped(64)),
                ));
                state = false;
            } else {
                rprintln!("send note on");
                let _ = midi.send_message(UsbMidiEventPacket::from_midi(
                    CableNumber::Cable0,
                    NoteOn(Channel::Channel1, Note::C2, U7::from_clamped(64)),
                ));
                state = true;
            }
            rprintln!("poll");
            if !usb_dev.poll(&mut [&mut midi]) {
                continue;
            }
            rprintln!("after poll");
            cortex_m::asm::delay(200_000_000); // about 1 s
        }
        rprintln!("un configured");
    }
}

pub const USB_CLASS_NONE: u8 = 0x00;
