#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::fs::{metadata, File};
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
    //println!("{:?}", memory);
    fetch_opcode(&memory, &mut program_counter);
}

fn rst(mem: &mut [u8; 0xFFF], adr: &mut u16, pc: &mut u16) {
    *adr = 0;
    // the CHIP-8 interpreter itself occupies the first 512 bytes of the memory space
    *pc = 0x200;

    // load chip-8 rom from file to mem array
    let mut f: File = File::open(r"C:\Users\Danyaal\Documents\Rust\chip8\test_opcode.ch8")
        .expect("could not read file");
    f.read(&mut mem[0x200..0xFFF])
        .expect("could not write bytes to buffer");
}

fn fetch_opcode(mem: &[u8; 0xFFF], pc: &mut u16) -> u16 {
    let mut opc: u16;
    // combine 2 BYTEs from memory into a WORD
    opc = mem[(*pc + 1) as usize] as u16;
    opc <<= 8;
    // bitwise or is actually the same as add here since opc && (mem[(*pc + 1) as usize] as u16) == 0
    opc |= mem[(*pc + 1) as usize] as u16;
    println!("{:b}", opc);
    opc
}

fn decode_opcode(opcode: u16) {
    // NNN:   address
    // NN:    8-bit const
    // N:     4-bit const
    // X, Y:  4-bit register identifier
    // PC:    Program Counter
    // VN:    One of 16 vars. N is 0x0-F
    // I:     16-bit addr reg (akin to void ptr)
    match opcode & 0xF000 {
        // 00E0, 00EE
        0x0000 => match opcode & 0x000F {
            0x0000 => op_00E0(),
            0x000E => op_00EE(),
            _ => unreachable!(),
        },
        // 1NNN
        0x1000 => op_1NNN(),
        // 2NNN
        0x2000 => op_2NNN(),
        // 3XNN
        0x3000 => op_3XNN(),
        // 4XNN
        0x4000 => op_4XNN(),
        // 5XY0
        0x5000 => op_5XY0(),
        // 6XNN
        0x6000 => op_6XNN(),
        // 7XNN
        0x7000 => op_7XNN(),
        // 8XYN
        0x8000 => match opcode & 0x000F {
            0x0000 => op_8XY0(),
            0x0001 => op_8XY1(),
            0x0002 => op_8XY2(),
            0x0003 => op_8XY3(),
            0x0004 => op_8XY4(),
            0x0005 => op_8XY5(),
            0x0006 => op_8XY6(),
            0x0007 => op_8XY7(),
            0x000E => op_8XYE(),
            _ => unreachable!(),
        },
        // 9XY0
        0x9000 => op_9XY0(),
        0xA000 => op_ANNN(),
        0xB000 => op_BNNN(),
        0xC000 => op_CXNN(),
        0xD000 => op_DXYN(),
        0xE000 => match opcode & 0x00FF {
            0x009E => op_EX9E(),
            0x00A1 => op_EXA1(),
            _ => unreachable!(),
        },
        0xF000 => match opcode & 0x00FF {
            0x0007 => op_FX07(),
            0x000A => op_FX0A(),
            0x0015 => op_FX15(),
            0x0018 => op_FX18(),
            0x001E => op_FX1E(),
            0x0029 => op_FX29(),
            0x0033 => op_FX33(),
            0x0055 => op_FX55(),
            0x0065 => op_FX65(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
    ()
}

fn op_00E0() {}
fn op_00EE() {}
fn op_1NNN() {}
fn op_2NNN() {}
fn op_3XNN() {}
fn op_4XNN() {}
fn op_5XY0() {}
fn op_6XNN() {}
fn op_7XNN() {}
fn op_8XY0() {}
fn op_8XY1() {}
fn op_8XY2() {}
fn op_8XY3() {}
fn op_8XY4() {}
fn op_8XY5() {}
fn op_8XY6() {}
fn op_8XY7() {}
fn op_8XYE() {}
fn op_9XY0() {}
fn op_ANNN() {}
fn op_BNNN() {}
fn op_CXNN() {}
fn op_DXYN() {}
fn op_EX9E() {}
fn op_EXA1() {}
fn op_FX07() {}
fn op_FX0A() {}
fn op_FX15() {}
fn op_FX18() {}
fn op_FX1E() {}
fn op_FX29() {}
fn op_FX33() {}
fn op_FX55() {}
fn op_FX65() {}
