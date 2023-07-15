use std::{io::Write};

use crate::cpu::device::Device;
use colored::*;

pub struct Port {
    /// inclusive
    pub begin: usize,
    /// exclusive
    pub end: usize,
    pub device: Box<dyn Device>
}

impl Port {
    pub fn new<T: Device + 'static>(begin: usize, end: usize, device: T) -> Self {
        Self {
            device: Box::new(device),
            begin, end
        }
    }
}

pub struct CPU {
    pub rx: u8,
    pub ry: u8,
    pub rz: u8,
    pub ip: u8,
    pub stack: Vec<u8>,
    ports: Vec<Port>
}

fn format_byte_as_ascii(byte: &u8) -> char {
    if *byte >= 0x20 && *byte <= 0x7e {
        char::try_from(*byte).unwrap()
    } else { '.' }
}

impl CPU {
    pub fn new(ports: Vec<Port>) -> Self {
        Self {
            rx: 0,
            ry: 0,
            rz: 0,
            ip: 0,
            stack: vec![],
            ports
        }
    }
    
    fn map_address(&mut self, address: u8) -> Option<(&mut dyn Device, u8)> {
        for port in &mut self.ports {
            if address >= port.begin as u8 && (address as usize) < port.end {
                return Some((port.device.as_mut(), address - port.begin as u8));
            }
        }

        return None
    }

    pub fn read(&mut self, address: u8) -> u8 {
        let (device, address) = match self.map_address(address) {
            Some(pair) => pair,
            None => return 0
        };

        device.read(address)
    }

    fn write(&mut self, address: u8, data: u8) {
        let (device, address) = match self.map_address(address) {
            Some(pair) => pair,
            None => return
        };

        device.write(address, data);
    }

    fn advance(&mut self) {
        self.ip = self.ip.wrapping_add(1);
    }

    fn im(&mut self) -> u8 {
        self.advance();
        self.read(self.ip)
    }

    fn execute_instruction(&mut self) {
        let instruction = self.read(self.ip);
        match instruction {
            // nop
            0x00 => {
                self.advance();
            }
            // put rx,<im>
            0x01 => {
                self.rx = self.im();

                self.advance();
            }
            // str <im>,rx
            0x02 => {
                let dest = self.im();

                self.write(dest, self.rx);

                self.advance();
            }
            // hlt
            0x03 => {
                loop {}
            }
            // ldr rx,<im>
            0x04 => {
                let src = self.im();

                self.rx = self.read(src);

                self.advance();
            }
            // put ry,<im>
            0x05 => {
                self.ry = self.im();

                self.advance();
            }
            // ldr rx,ry
            0x06 => {
                self.rx = self.read(self.ry);

                self.advance();
            }
            // jpz <im>,rx
            0x07 => {
                let dest = self.im();

                if self.rx == 0 {
                    self.ip = dest;
                } else {
                    self.advance();
                }
            }
            // inc ry
            0x08 => {
                self.ry = self.ry.wrapping_add(1);

                self.advance();
            }
            // jmp <im>
            0x09 => {
                self.ip = self.im();
            }
            // inc rx
            0x0a => {
                self.rx = self.rx.wrapping_add(1);

                self.advance();
            }
            // sub ry,rx
            0x0b => {
                self.ry = self.ry.wrapping_sub(self.rx);

                self.advance();
            }
            // jpz <im>,ry
            0x0c => {
                let dest = self.im();

                if self.ry == 0 {
                    self.ip = dest;
                } else {
                    self.advance();
                }
            }
            // ldr ry,<im>
            0x0d => {
                let src = self.im();

                self.ry = self.read(src);

                self.advance();
            }
            // sub rx,ry
            0x0e => {
                self.rx = self.rx.wrapping_sub(self.ry);

                self.advance();
            }
            // add rx,ry
            0x0f => {
                self.rx = self.rx.wrapping_add(self.ry);

                self.advance();
            }
            // str rx,ry
            0x10 => {
                self.write(self.rx, self.ry);

                self.advance();
            }
            // str <im>,ry
            0x11 => {
                let dest = self.im();

                self.write(dest, self.ry);

                self.advance();
            }
            // jpn <im>,rx
            0x12 => {
                let dest = self.im();

                if self.rx & 0b10000000 != 0 {
                    self.ip = dest;
                } else {
                    self.advance();
                }
            }
            // psh <im>
            0x13 => {
                let value = self.im();

                self.stack.push(value);

                self.advance();
            }
            // put rz,<im>
            0x14 => {
                self.rz = self.im();

                self.advance();
            }
            // inc rz
            0x15 => {
                self.rz = self.rz.wrapping_add(1);

                self.advance();
            }
            // pop ry
            0x16 => {
                self.ry = self.stack.pop().unwrap_or(0);

                self.advance();
            }
            // jmp ry
            0x17 => {
                self.ip = self.ry;
            }
            // psh rx
            0x18 => {
                self.stack.push(self.rx);

                self.advance();
            }
            // jpz <im>,rz
            0x19 => {
                let dest = self.im();

                if self.rz == 0 {
                    self.ip = dest;
                } else {
                    self.advance();
                }
            }
            // put rx,rz
            0x1a => {
                self.rx = self.rz;

                self.advance();
            }
            // pop rx
            0x1b => {
                self.rx = self.stack.pop().unwrap_or(0);

                self.advance();
            }
            // jmp rx
            0x1c => {
                self.ip = self.rx;
            }
            // ldr ry,rx
            0x1d => {
                self.ry = self.read(self.rx);

                self.advance();
            }
            // add rz,rx
            0x1e => {
                self.rz = self.rz.wrapping_add(self.rx);

                self.advance();
            }
            // dec ry
            0x1f => {
                self.ry = self.ry.wrapping_sub(1);

                self.advance();
            }
            // str <im>,rz
            0x20 => {
                let dest = self.im();

                self.write(dest, self.rz);

                self.advance();
            }
            // sub ry,rz
            0x21 => {
                self.ry = self.ry.wrapping_sub(self.rz);

                self.advance();
            }
            // add ry,rz
            0x22 => {
                self.ry = self.ry.wrapping_add(self.rz);

                self.advance();
            }
            // put rx,ry
            0x23 => {
                self.rx = self.ry;

                self.advance();
            }
            // psh ry
            0x24 => {
                self.stack.push(self.ry);

                self.advance();
            }
            // jpn <im>,ry
            0x25 => {
                let dest = self.im();

                if self.ry & 0b10000000 != 0 {
                    self.ip = dest;
                } else {
                    self.advance();
                }
            }
            // dec rx
            0x26 => {
                self.rx = self.rx.wrapping_sub(1);

                self.advance();
            }
            // pop rz
            0x27 => {
                self.rz = self.stack.pop().unwrap_or(0);

                self.advance();
            }
            // jmp rz
            0x28 => {
                self.ip = self.rz;
            }
            // sub rz,rx
            0x29 => {
                self.rz = self.rz.wrapping_sub(self.rx);

                self.advance();
            }
            // dec rz
            0x2a => {
                self.rz = self.rz.wrapping_sub(1);

                self.advance();
            }
            // neg rx
            0x2b => {
                self.rx = (self.rx as i8).wrapping_neg() as u8;

                self.advance();
            }
            // neg ry
            0x2c => {
                self.ry = (self.ry as i8).wrapping_neg() as u8;

                self.advance();
            }
            // neg rz
            0x2d => {
                self.rz = (self.ry as i8).wrapping_neg() as u8;

                self.advance();
            }
            // ret
            // EXPERIMENTAL: equivalet to
            //  pop <reg>
            //  jmp <reg>
            0x2e => {
                self.ip = self.stack.pop().unwrap_or(0);
            }
            // put rz,rx
            0x2f => {
                self.rz = self.rx;

                self.advance();
            }
            // psh rz
            0x30 => {
                self.stack.push(self.rz);

                self.advance();
            }
            // sub rx,rz
            0x31 => {
                self.rx = self.rx.wrapping_sub(self.rz);

                self.advance();
            }
            // ldr rz,<im>
            0x32 => {
                let src = self.im();

                self.rz = self.read(src);

                self.advance();
            }
            // and rz,<im>
            0x33 => {
                let src = self.im();

                self.rz &= src;

                self.advance();
            }
            // put ry,rx
            0x34 => {
                self.ry = self.rx;

                self.advance();
            }
            // and ry,<im>
            0x35 => {
                let src = self.im();

                self.ry &= src;

                self.advance();
            }
            // add ry,rx
            0x36 => {
                self.ry = self.ry.wrapping_add(self.rx);

                self.advance();
            }
            // sub rz,<im>
            0x37 => {
                let src = self.im();

                self.rz = self.rz.wrapping_sub(src);

                self.advance();
            }
            // add rz,<im>
            0x38 => {
                let src = self.im();

                self.rz = self.rz.wrapping_add(src);

                self.advance();
            }
            // sub rz,ry
            0x39 => {
                self.rz = self.rz.wrapping_sub(self.ry);

                self.advance();
            }
            // add rx,rz
            0x3a => {
                self.rx = self.rx.wrapping_add(self.rz);

                self.advance();
            }
            // and rx,<im>
            0x3b => {
                let src = self.im();

                self.rx &= src;

                self.advance();
            }
            // add rz,ry
            0x3c => {
                self.rz = self.rz.wrapping_add(self.ry);

                self.advance();
            }
            // or  rx,ry
            0x3d => {
                self.rx |= self.ry;

                self.advance();
            }
            // str rz,rx
            0x3e => {
                self.write(self.rz, self.rx);

                self.advance();
            }
            // ldr ry,ry
            0x3f => {
                self.ry = self.read(self.ry);

                self.advance();
            }
            _    => {
                println!("{:#x?}", instruction);
                unimplemented!()
            }
        }
    }

    fn peak_all(&mut self) -> [u8; 256] {
        let mut out = [0; 256];

        for i in 0..=255u8 {
            out[i as usize] = match self.map_address(i) {
                Some((device, address)) => device.peak(address),
                None => 0
            };
        }

        out
    }

    pub fn step(&mut self) {
        self.execute_instruction();
    }

    pub fn print_address_space<T: Write>(&mut self, out: &mut T) {
        let array = self.peak_all();

        writeln!(out, "{}", "    00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f  decoded text\n".bright_black()).unwrap();
    
        for i in (0..255usize).step_by(16) {
            let decoded: Vec<String> = array[i..i + 16].iter()
                .map(|byte| {
                    String::from(format_byte_as_ascii(byte))
                })
                .collect();
            write!(out, "{}  ", format!("{:02x}", i).bright_black()).unwrap();

            for string in array[i..i + 16].iter()
                .enumerate()
                .map(|(o, byte)| {
                    let out = format!("{:02x}", byte);

                    if i + o == self.ip as usize {
                        out.black().on_white()
                    } else { out.white() }
                }) {
                write!(out, "{} ", string).unwrap();
            }

            writeln!(out, " {}", decoded.join(" ")).unwrap();
        }
    }
}