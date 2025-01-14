#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate panic_halt;

use alloc::boxed::Box;
use core::marker::{Send, Sync};
use core::{
    cell::RefCell,
    concat,
    default::Default,
    format_args,
    ops::DerefMut,
    option::{Option, Option::*},
    result::Result::*,
};
use cortex_m::{
    asm::{dmb, dsb},
    interrupt::Mutex,
    peripheral::NVIC,
};

use rtt_target::rprintln;
use stm32f1xx_hal::{
    // gpio::{self, OpenDrain, Output, PinState},
    pac,
    pac::interrupt,
    prelude::*,
    serial::{Config, Rx, Serial as Hal_Serial, Tx},
};

// type RedLed = gpio::Pin<'C', 7, Output<OpenDrain>>;

// static WAKE_LED: Mutex<RefCell<Option<RedLed>>> = Mutex::new(RefCell::new(None));

pub trait RXProcessor: Send + Sync {
    fn process_character(&'static self, character: u8);
}

static RX: Mutex<RefCell<Option<Rx<pac::USART2>>>> = Mutex::new(RefCell::new(None));
static RX_PROCESSOR: Mutex<RefCell<Option<Box<&dyn RXProcessor>>>> = Mutex::new(RefCell::new(None));

#[repr(C)]
pub struct Serial {
    tx: Tx<pac::USART2>,
}

/// <div rustbindgen nocopy></div>
/// <div rustbindgen opaque></div>
pub struct Board {
    pub serial: Serial,
}

impl Board {
    pub fn new() -> Self {
        // Get access to the device specific peripherals from the peripheral access crate
        let p = pac::Peripherals::take().unwrap();

        // Take ownership over the raw flash and rcc devices and convert them into the corresponding
        // HAL structs
        let mut flash = p.FLASH.constrain();
        let rcc = p.RCC.constrain();

        // Freeze the configuration of all the clocks in the system
        // and store the frozen frequencies in `clocks`
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Prepare the alternate function I/O registers
        let mut afio = p.AFIO.constrain();

        // Prepare the peripherals
        let mut gpioa = p.GPIOA.split();
        // let mut gpioc = p.GPIOC.split();

        // USART2
        let tx_pin = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
        let rx_pin = gpioa.pa3;

        // Init Serial
        rprintln!("initializing serial");
        let mut serial = Hal_Serial::new(
            p.USART2,
            (tx_pin, rx_pin),
            &mut afio.mapr,
            Config::default().baudrate(115200.bps()),
            &clocks,
        );

        rprintln!("serial rx.listen()");
        serial.rx.listen();
        serial.rx.listen_idle();

        // let led = gpioc
        //     .pc7
        //     .into_open_drain_output_with_state(&mut gpioc.crl, PinState::Low);

        cortex_m::interrupt::free(|cs| {
            RX.borrow(cs).replace(Some(serial.rx));
            // WAKE_LED.borrow(cs).replace(Some(led));
        });

        rprintln!("unmasking USART2 interrupt");
        unsafe {
            NVIC::unmask(pac::Interrupt::USART2);
        }

        Board {
            serial: Serial { tx: serial.tx },
        }
    }

    pub fn set_rx_processor(&mut self, processor: Box<&'static dyn RXProcessor>) {
        cortex_m::interrupt::free(|cs| {
            let mut global_rx_binding = RX_PROCESSOR.borrow(cs).borrow_mut();
            *global_rx_binding = Some(processor);
        });
    }
}

#[interrupt]
unsafe fn USART2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut rx) = RX.borrow(cs).borrow_mut().deref_mut() {
            if rx.is_rx_not_empty() {
                // if let Some(led) = WAKE_LED.borrow(cs).borrow_mut().deref_mut() {
                //     led.is_set_low();
                // }
                dsb();
                if let Ok(c) = nb::block!(rx.read()) {
                    rprintln!("serial rx byte: {}", c);
                    let r = RX_PROCESSOR.borrow(cs);

                    if let Some(processor) = r.borrow_mut().deref_mut() {
                        processor.process_character(c);
                    }
                }
                dmb();
                // if let Some(led) = WAKE_LED.borrow(cs).borrow_mut().deref_mut() {
                //     led.is_set_high();
                // }
                rx.listen_idle();
            }
        }
    })
}
