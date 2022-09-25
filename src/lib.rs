#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(never_type)]

mod rgb;
mod keypad;
pub mod usb;

pub use rgb::{Rgb, NUM_LEDS};
pub use keypad::Keypad;


type UsbChan = &'static embassy_sync::channel::Channel<embassy_sync::blocking_mutex::raw::ThreadModeRawMutex, (usize, [u8; 64]), 1>;