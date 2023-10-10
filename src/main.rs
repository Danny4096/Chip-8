#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::vec;

fn main() {
    // the chip-8 uses 8 bit data registers and 16 bit address reg., pc, & stack
    type BYTE = u8;
    type WORD = u16;

    // the chip-8 has 0xFFF bytes of memory
    let mut memory: [BYTE; 0xFFF] = [0b00000000; 0xFFF];
    // 16 8-bit registers
    let mut registers: [BYTE; 0x10] = [0b00000000; 0x10];
    // 16 bit adr reg I
    let mut address_i: WORD = 0b0000000000000000;
    // 16 bit PC
    let mut program_counter: WORD = 0b0000000000000000;
    // stack
    let mut stack: Vec<WORD> = Vec::new();

    rst(&mut memory, &mut address_i, &mut program_counter);

    // the screen is 64*32
    // sprites are drawn using a co-ordinate system
    // sprites have a width of 8 and variable height up to 15
    // co-ords refer to the top-left pixel of a sprite
    let screen_buffer: [[BYTE; 64]; 32] = [[0; 64]; 32];
    println!("{:?}", screen_buffer);
    println!("{:?}", screen_buffer.len());
}

fn rst(mem: &mut [u8; 0xFFF], adr: &mut u16, pc: &mut u16) {
    *adr = 0;
    // the CHIP-8 interpreter itself occupies the first 512 bytes of the memory space
    *pc = 0x200;

    // load chip-8 rom from file to mem array
    let mut f = File::open(r"C:\Users\Danyaal\Documents\Rust\chip8\test_opcode.ch8")
        .expect("could not read file");
    f.read(&mut *mem).expect("could not write bytes to buffer");
}
