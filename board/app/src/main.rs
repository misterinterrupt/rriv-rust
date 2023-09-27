#![cfg_attr(not(test), no_std)]
#![allow(clippy::empty_loop)]
#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_main]

extern crate panic_halt;
use core::{prelude::rust_2024::*, u8};
use cortex_m_rt::entry;
use rtt_target::rtt_init_print;

pub mod prelude;

extern crate rriv_0_4;
use rriv_0_4::Board;

extern crate datalogger;
use datalogger::DataLogger;

// #[link(name = "rriv_0_4", kind = "static")]
// extern "C" {
//     pub fn rust_serial_interface_new() -> *mut c_void;
// }

#[entry]
fn main() -> ! {
    rtt_init_print!();
    prelude::init();

    let mut board = Board::new();
    let mut datalogger = DataLogger::new();
    datalogger.setup(&mut board);
    loop {
        datalogger.run_loop_iteration(& board);
    }
}

// some serial commands to test for success and failure
// {"cmd":"set","object":"datalogger"}
// {"cmd":"foo","object":"bar"}
