mod devices;
pub mod sti;

use std::fmt::Debug;

pub use devices::*;

pub trait Device: Debug {
    fn read(&mut self, address: u8) -> u8;
    fn write(&mut self, address: u8, data: u8);
    fn peak(&self, address: u8) -> u8;
}