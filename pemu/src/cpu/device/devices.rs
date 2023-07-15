use std::{io::{Write, Read}, fmt::Debug};
use byteorder::{WriteBytesExt, ReadBytesExt};
use crate::cpu::KeyboardOutput;
use super::Device;


#[derive(Debug)]
pub struct RAM {
    data: Vec<u8>
}

impl From<Vec<u8>> for RAM {
    fn from(value: Vec<u8>) -> Self {
        Self { data: value }
    }
}

impl Device for RAM {
    fn read(&mut self, address: u8) -> u8 {
        self.peak(address)
    }

    fn write(&mut self, address: u8, data: u8) {
        self.data[address as usize] = data
    }

    fn peak(&self, address: u8) -> u8 {
        self.data[address as usize]
    }
}


pub struct Stdout<'a> {
    stream: &'a mut dyn Write
}

impl<'a> Debug for Stdout<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Terminal")
    }
}

impl<'a> Device for Stdout<'a> {
    fn read(&mut self, address: u8) -> u8 {
        self.peak(address)
    }

    fn write(&mut self, _: u8, data: u8) {
        self.stream.write_u8(data).unwrap();
    }

    fn peak(&self, _: u8) -> u8 { 0 }
}


pub struct Stdin<'a> {
    input: &'a mut dyn Read,
    buffer: u8
}

impl<'a> Debug for Stdin<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Stdin")
    }
}

impl<'a> Stdin<'a> {
    pub fn new(input: &'a mut dyn Read) -> Self {
        Self { input, buffer: 0 }
    }
}

impl<'a> Device for Stdin<'a> {
    fn read(&mut self, address: u8) -> u8 {
        self.buffer = self.input.read_u8().unwrap();
        self.peak(address)
    }

    fn write(&mut self, _: u8, _: u8) { }

    fn peak(&self, _: u8) -> u8 {
        self.buffer
    }
}


#[derive(Debug)]
pub struct Floppy {
    address: u8,
    data: [u8; 256]
}

impl Floppy {
    pub fn new() -> Self {
        Self {
            address: 0,
            data: [0; 256]
        }
    }
}

impl Device for Floppy {
    fn read(&mut self, address: u8) -> u8 {
        self.peak(address)
    }

    fn write(&mut self, address: u8, data: u8) {
        match address {
            0 => self.address = data,
            1 => self.data[self.address as usize] = data,
            _ => unreachable!()
        }
    }

    fn peak(&self, address: u8) -> u8 {
        match address {
            0 => 0,
            1 => self.data[self.address as usize],
            _ => unreachable!()
        }
    }
}


#[derive(Debug)]
pub struct Keyboard {
    output: KeyboardOutput
}

impl Keyboard {
    pub fn new(output: KeyboardOutput) -> Self {
        Self { output }
    }
}

impl Device for Keyboard {
    fn read(&mut self, address: u8) -> u8 {
        self.peak(address)
    }

    fn write(&mut self, _: u8, _: u8) { }

    fn peak(&self, _: u8) -> u8 {
        *self.output.lock()
    }
}