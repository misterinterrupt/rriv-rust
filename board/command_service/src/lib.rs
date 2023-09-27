#![cfg_attr(not(test), no_std)]

use control_interface::command_recognizer::{CommandData, CommandRecognizer};
use control_interface::command_registry::{Command, CommandRegistry};
use rriv_board::{RRIVBoard, RXProcessor};

extern crate alloc;
use alloc::boxed::Box;

use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::ffi::{c_char, CStr};
use core::iter::Empty;
use core::ops::{Deref, DerefMut};
use core::ptr::null;
use cortex_m::interrupt::Mutex;
use serde::{Deserialize, Serialize};

use rtt_target::rprintln;

// static COMMAND_DATA: Mutex<RefCell<Option<CommandData>>> = Mutex::new(RefCell::new(None));
static mut COMMAND_DATA: CommandData = CommandData::default();
// static COMMAND_DATA: Option<CommandData> = None;
// static SETUP_DONE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false)); // unused and unnecessary

pub struct CommandService {
    registry: CommandRegistry,
}

#[derive(Serialize, Deserialize, Debug)]
struct CLICommand<'a> {
    object: &'a str,
    action: &'a str,
}

impl CommandService {
    pub fn new() -> Self {
        // set the static, shareable command data
        CommandService {
            registry: CommandRegistry::new(),
        }
    }

    /// set the global rx processor
    pub fn setup(&mut self, board: &mut impl RRIVBoard) {
        let char_processor = Box::<CharacterProcessor>::leak(Box::new(
            CharacterProcessor::new(),
        ));

        // pass a pointer to the lleaked processor to Board::set_rx_processor
        board.set_rx_processor(Box::new(char_processor));
    }

    /// register a command with two &strs, object and action, and a C function pointer that matches registry.register_command's second argument
    /// this calls registry.get_command_from_parts to get a Command object, then calls registry.register_command
    pub fn register_command(
        &mut self,
        object: &str,
        action: &str,
        ffi_cb: extern "C" fn(*const c_char),
    ) {
        let command = self.registry.get_command_from_parts(object, action);
        self.registry.register_command(command, ffi_cb);
    }

    fn pending_message_count(&self) -> usize {
        // use critical section, don't need the Mutex and instead use unsafe{}
        cortex_m::interrupt::free(|cs| unsafe {
            let command_data = COMMAND_DATA.borrow_mut();
            CommandRecognizer::pending_message_count(&command_data)
        })
    }

    fn take_command(&mut self) -> Result<[u8; 100], ()> {
        cortex_m::interrupt::free(|cs| {
            unsafe {
                let command_data = COMMAND_DATA.borrow_mut();
                Ok(CommandRecognizer::take_command(command_data))
            }
        })
    }

    pub fn run_loop_iteration(&mut self) {
        if self.pending_message_count() > 0 {
            if let Ok(command) = self.take_command() {
                self.handle_serial_command(command);
            }
        }
    }

    fn handle_serial_command(&self, serial_command_bytes: [u8; 100]) {
        let command_data_cstr = CStr::from_bytes_until_nul(&serial_command_bytes).unwrap();
        let command_data_str = command_data_cstr.to_str().unwrap();
        // Parse the JSON string into a serde_json::Value
        rprintln!("{}", command_data_str);

        // Next
        // 2. port the EEPROM
        // 3. add and remove configurations from EEPROM

        match serde_json::from_str::<CLICommand>(command_data_str) {
            Ok(cli_command) => {
                let command = self
                    .registry
                    .get_command_from_parts(cli_command.object, cli_command.action);
                self.execute_command(command, command_data_cstr);
            }
            Err(_) => self.execute_command(Command::Unknown, command_data_cstr),
        }
    }

    fn execute_command(&self, command: Command, command_cstr: &CStr) {
        // check if there is a function pointer registered for this command
        if let Some(ffi_cb) = self.registry.get_action_fn(command) {
            // call the registered function pointer and pass ownership of the command bytes to C
            ffi_cb(command_cstr.as_ptr() as *mut c_char);
        } else if let Some(unknown_cb) = self.registry.get_action_fn(Command::Unknown) {
            // send a message back to the host that the command was not recognized
            unknown_cb(command_cstr.as_ptr() as *mut c_char);
        }
    }
}

/// I believe the lifetimes here are generic over the lifetime of the CommandData
pub struct CharacterProcessor { }

impl<'a> CharacterProcessor {
    pub fn new() -> CharacterProcessor {
        CharacterProcessor {  }
    }
}

impl<'a, 'b> RXProcessor for CharacterProcessor {
    fn process_character(&self, character: u8) {
        cortex_m::interrupt::free(|cs| {
            unsafe {
                let command_data = COMMAND_DATA.borrow_mut();
                CommandRecognizer::process_character(command_data, character);
            }
        })
    }
}
