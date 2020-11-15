//! USB OTG full-speed peripheral
//!
//! Requires the `usb_fs` feature.
//! Only one of the `usb_fs`/`usb_hs` features can be selected at the same time.

use crate::pac;

use crate::gpio::{
    gpiob::{PB14, PB15},
    Alternate, AF12,
};
use crate::rcc::Clocks;
use crate::time::Hertz;

pub use synopsys_usb_otg::UsbBus;
use synopsys_usb_otg::{PhyType, UsbPeripheral};

pub struct USB {
    // pub usb_phy: pac::USBPHYC, // later
    pub usb_global: pac::OTG_HS_GLOBAL,
    pub usb_device: pac::OTG_HS_DEVICE,
    pub usb_pwrclk: pac::OTG_HS_PWRCLK,
    pub pin_dm: PB14<Alternate<AF12>>,
    pub pin_dp: PB15<Alternate<AF12>>,
    pub hclk: Hertz,
}

impl USB {
    /// Construct a USB peripheral wrapper.
    ///
    /// Call `UsbBus::new` to construct and initialize the USB peripheral driver.
    pub fn new(
        // usb_phy: pac::USBPHYC, // later
        usb_global: pac::OTG_HS_GLOBAL,
        usb_device: pac::OTG_HS_DEVICE,
        usb_pwrclk: pac::OTG_HS_PWRCLK,
        pins: (PB14<Alternate<AF12>>, PB15<Alternate<AF12>>),
        clocks: Clocks,
    ) -> Self {
        Self {
            usb_global,
            usb_device,
            usb_pwrclk,
            pin_dm: pins.0,
            pin_dp: pins.1,
            hclk: clocks.hclk(),
        }
    }
}

unsafe impl Sync for USB {}

unsafe impl UsbPeripheral for USB {
    const REGISTERS: *const () = pac::OTG_HS_GLOBAL::ptr() as *const ();

    const HIGH_SPEED: bool = true;
    const FIFO_DEPTH_WORDS: usize = 1024;
    const ENDPOINT_COUNT: usize = 9;

    fn enable() {
        let rcc = unsafe { &*pac::RCC::ptr() };

        cortex_m::interrupt::free(|_| {
            // Enable USB peripheral
            rcc.ahb1enr.modify(|_, w| w.otghsen().set_bit());

            // Reset USB peripheral
            rcc.ahb1rstr.modify(|_, w| w.otghsrst().set_bit());
            rcc.ahb1rstr.modify(|_, w| w.otghsrst().clear_bit());

            // Enable and reset HS Phy
            rcc.ahb1enr.modify(|_, w| w.otghsulpien().enabled());
            rcc.apb2enr.modify(|_, w| w.usbphycen().enabled());
            rcc.apb2rstr.modify(|_, w| w.usbphycrst().reset());
            rcc.apb2rstr.modify(|_, w| w.usbphycrst().clear_bit());
        });
    }

    fn ahb_frequency_hz(&self) -> u32 {
        self.hclk.0
    }

    #[inline(always)]
    fn phy_type(&self) -> PhyType {
        PhyType::InternalHighSpeed
    }

    // Setup LDO and PLL
    fn setup_internal_hs_phy(&self) {
        let phy = unsafe { &*pac::USBPHYC::ptr() };

        // Turn on LDO
        // For some reason setting the bit enables the LDO
        phy.ldo.modify(|_, w| w.ldo_disable().set_bit());

        // Busy wait until ldo_status becomes true
        // Notice, this may hang
        while phy.ldo.read().ldo_status().bit_is_clear() {}

        // Setup PLL
        // This disables the the pll1 during tuning
        phy.pll1.write(|w| unsafe {
            w.pll1sel().bits(0b101 /* A value for 25MHz HSE */)
        });

        phy.tune.modify(|r, w| unsafe { w.bits(r.bits() | 0xF13) });

        phy.pll1.modify(|_, w| w.pll1en().set_bit());

        // 2ms Delay required to get internal phy clock stable
        cortex_m::asm::delay(432000);
    }
}

pub type UsbBusType = UsbBus<USB>;
