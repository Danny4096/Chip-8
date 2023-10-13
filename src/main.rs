#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::{thread_rng, Rng};
use std::fs::{metadata, File};
use std::io;
use std::io::prelude::*;
use std::vec;

fn main() {
    // the chip-8 uses 8 bit data registers and 16 bit address reg., pc, & stack
    type BYTE = u8;
    type WORD = u16;

    // the chip-8 has 0xFFF bytes of memory
    let mut memory: [BYTE; 0xFFF] = [0x00; 0xFFF];
    // 16 8-bit registers
    let mut registers: [BYTE; 0x10] = [0x00; 0x10];
    // 16 bit adr reg I
    let mut address_i: WORD = 0x0000;
    // 16 bit PC
    let mut program_counter: WORD = 0x0000;
    // stack
    let mut stack: Vec<WORD> = Vec::new();

    rst(&mut memory, &mut address_i, &mut program_counter);
    store_hex_digits(&mut memory);

    // the screen is 64*32
    // sprites are drawn using a co-ordinate system
    // sprites have a width of 8 and variable height up to 15
    // co-ords refer to the top-left pixel of a sprite
    let mut screen_buffer: [[BYTE; 64]; 32] = [[0; 64]; 32];
    println!("{:?}", memory);
    for _ in 0..50 {
        let opcode: u16 = fetch_opcode(&memory, &mut program_counter);
        println!("{:x}", opcode);
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
            0x1000 => op_1NNN!(program_counter, opcode),
            // 2NNN
            0x2000 => op_2NNN!(program_counter, opcode, stack),
            // 3XNN
            0x3000 => op_3XNN!(program_counter, opcode, registers),
            // 4XNN
            0x4000 => op_4XNN!(program_counter, opcode, registers),
            // 5XY0
            0x5000 => op_5XY0!(program_counter, opcode, registers),
            // 6XNN
            0x6000 => op_6XNN!(opcode, registers),
            // 7XNN
            0x7000 => op_7XNN!(opcode, registers),
            // 8XYN
            0x8000 => match opcode & 0x000F {
                0x0000 => op_8XY0!(opcode, registers),
                0x0001 => op_8XY1!(opcode, registers),
                0x0002 => op_8XY2!(opcode, registers),
                0x0003 => op_8XY3!(opcode, registers),
                0x0004 => op_8XY4!(opcode, registers),
                0x0005 => op_8XY5!(opcode, registers),
                0x0006 => op_8XY6!(opcode, registers),
                0x0007 => op_8XY7!(opcode, registers),
                0x000E => op_8XYE!(opcode, registers),
                _ => unreachable!(),
            },
            // 9XY0
            0x9000 => op_9XY0!(program_counter, opcode, registers),
            0xA000 => op_ANNN!(address_i, opcode),
            0xB000 => op_BNNN!(program_counter, opcode, registers),
            0xC000 => op_CXNN!(opcode, registers),
            0xD000 => op_DXYN!(address_i, opcode, registers, screen_buffer, memory),
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
                0x001E => op_FX1E!(address_i, opcode, registers),
                0x0029 => op_FX29!(address_i, opcode, registers),
                0x0033 => op_FX33!(address_i, opcode, registers, memory),
                0x0055 => op_FX55!(address_i, opcode, registers, memory),
                0x0065 => op_FX65!(address_i, opcode, registers, memory),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
        println!("{:x}", program_counter);
    }
}

fn rst(mem: &mut [u8; 0xFFF], adr: &mut u16, pc: &mut u16) {
    *adr = 0;
    // the CHIP-8 interpreter itself occupies the first 512 bytes of the memory space
    *pc = 0x200;

    // load chip-8 rom from file to mem array
    let mut f: File = File::open(r"C:\Users\Danyaal\Documents\Rust\chip8\1-chip8-logo.ch8")
        .expect("could not read file");
    f.read(&mut mem[0x200..0xFFF])
        .expect("could not write bytes to buffer");
}

fn store_hex_digits(mem: &mut [u8; 0xFFF]) {
    let digits: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, 0xF0, 0x90, 0x90, 0x90, 0xF0, 0xF0, 0x10, 0xF0, 0x80, 0xF0,
        0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
        0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0,
        0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
        0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0,
        0xF0, 0x80, 0xF0, 0x80, 0x80,
    ];
    let _ = &mut mem[0..0x50].clone_from_slice(&digits);
}

fn fetch_opcode(mem: &[u8; 0xFFF], pc: &mut u16) -> u16 {
    let mut opc: u16;
    // combine 2 BYTEs from memory into a WORD
    opc = mem[(*pc) as usize] as u16;
    opc <<= 8;
    // bitwise or is actually the same as add here since opc && (mem[(*pc + 1) as usize] as u16) == 0
    opc |= mem[(*pc + 1) as usize] as u16;
    println!("{:b}", opc);
    *pc += 2;
    opc
}

fn op_00E0() {}
fn op_00EE() {}

// jump to addr NNN
#[macro_export]
macro_rules! op_1NNN {
    ($pc:expr, $opcode:expr) => {
        $pc = $opcode & 0x0FFF
    };
}

// call subroutine at NNN
#[macro_export]
macro_rules! op_2NNN {
    ($pc:expr, $opcode:expr, $stack:expr) => {{
        $stack.push($pc);
        $pc = $opcode & 0x0FFF
    }};
}

// skip if Vx == NN
#[macro_export]
macro_rules! op_3XNN {
    ($pc:expr, $opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let n = ($opcode & 0x0F00) as u8;
        if $regs[regx] == n {
            $pc += 2;
        }
    }};
}

// skip if Vx != NN
#[macro_export]
macro_rules! op_4XNN {
    ($pc:expr, $opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let n = ($opcode & 0x0F00) as u8;
        if $regs[regx] != n {
            $pc += 2;
        }
    }};
}

// skip if Vx == Vy
#[macro_export]
macro_rules! op_5XY0 {
    ($pc:expr, $opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        if $regs[regx] == $regs[regy] {
            $pc += 2;
        }
    }};
}

// Vx = NN
#[macro_export]
macro_rules! op_6XNN {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let n = ($opcode & 0x00FF) as u8;
        $regs[regx] = n;
    }};
}

// Vx += NN
#[macro_export]
macro_rules! op_7XNN {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let n = ($opcode & 0x00FF) as u16;
        $regs[regx] = ($regs[regx] as u16 + n) as u8;
    }};
}

// Vx = Vy
#[macro_export]
macro_rules! op_8XY0 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[regx] = $regs[regy];
    }};
}

// Vx |= Vy
#[macro_export]
macro_rules! op_8XY1 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[regx] |= $regs[regy];
    }};
}

// Vx &= Vy
#[macro_export]
macro_rules! op_8XY2 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[regx] &= $regs[regy];
    }};
}

// Vx ^= Vy
#[macro_export]
macro_rules! op_8XY3 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[regx] ^= $regs[regy];
    }};
}

// Vx += Vy; VF = 1 if carry
#[macro_export]
macro_rules! op_8XY4 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        let res: u16 = $regs[regx] as u16 + $regs[regy] as u16;
        $regs[0xF] = if res > 0xFF { 1 } else { 0 };
    }};
}

// Vx -= Vy; VF = 1 if overflow
#[macro_export]
macro_rules! op_8XY5 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[0xF] = if $regs[regy] > $regs[regx] { 1 } else { 0 };
        $regs[regx] = $regs[regx].wrapping_sub($regs[regy]);
    }};
}

// store lsb and bit shift right
#[macro_export]
macro_rules! op_8XY6 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[0xF] = ($regs[regy] & 0b00000001) as u8;
        $regs[regx] = $regs[regy] >> 1;
    }};
}

// Vx = Vy - Vx
#[macro_export]
macro_rules! op_8XY7 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[0xF] = if $regs[regx] > $regs[regy] { 1 } else { 0 };
        $regs[regx] = $regs[regy].wrapping_sub($regs[regx]);
    }};
}

// store msb in VF and bit shift Vy left
#[macro_export]
macro_rules! op_8XYE {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        $regs[0xF] = ($regs[regy] & 0b10000000) as u8;
        $regs[regx] = $regs[regy] << 1;
    }};
}

// skip if  Vx != Vy
#[macro_export]
macro_rules! op_9XY0 {
    ($pc:expr, $opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        if $regs[regx] != $regs[regy] {
            $pc += 4;
        }
    }};
}

// store NNN in address_i
#[macro_export]
macro_rules! op_ANNN {
    ($adr_i:expr, $opcode:expr) => {
        $adr_i = $opcode & 0x0FFF
    };
}

// jump to adress NNN + V0s
#[macro_export]
macro_rules! op_BNNN {
    ($pc:expr, $opcode:expr, $regs:expr) => {
        $pc = ($opcode & 0x0FFF) + $regs[0] as u16
    };
}

// set Vx to a random number with a mask of NN
#[macro_export]
macro_rules! op_CXNN {
    ($opcode:expr, $regs:expr) => {{
        let mut rng = rand::thread_rng();
        let n: u8 = rng.gen();
        $regs[(($opcode & 0x0F00) >> 8) as usize] = n & ($opcode & 0x00FF) as u8;
    }};
}

// doodling time!!
// top left corner is (Vx, Vy)
// get n bytes from memory starting at address_i
// width is always 8 bits
#[macro_export]
macro_rules! op_DXYN {
    ($adr_i:expr, $opcode:expr, $regs:expr, $vram:expr, $mem:expr) => {{
        // decode x and y
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 8) as usize;
        let n = (($opcode & 0x000F) >> 8) as u8;
        for byte in 0..n {
            // top corner + current row of sprite (modded w screen height for wrapping)
            let y = (($regs[regy] + byte) % 32) as usize;
            let sprite_data = $mem[($adr_i + byte as u16) as usize];
            for bit in 0..8 {
                let x = (($regs[regy] + bit) % 64) as usize;
                let pixel = (sprite_data >> (7 - bit)) & 1;
                $regs[0xF] = $vram[x][y] ^ pixel;
                $vram[x][y] ^= pixel;
            }
        }
    }};
}

// input
fn op_EX9E() {}
fn op_EXA1() {}
fn op_FX07() {}
fn op_FX0A() {}

//delay timer
fn op_FX15() {}

// sound timer
fn op_FX18() {}

// adr_i += Vx
#[macro_export]
macro_rules! op_FX1E {
    ($adr_i:expr, $opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        $adr_i += $regs[regx] as u16;
    }};
}

// set adr_i to adr of sprite data for digit in VX

#[macro_export]
macro_rules! op_FX29 {
    ($adr_i:expr, $opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let char_index = $regs[regx] as u16;
        // font sprites are 5 bytes long and start at the start of RAM
        $adr_i = char_index * 5;
    }};
}
// BCD of Vx in I, I+1, I+2
// BCD is basically storing the placevalues of the decimal value separately
#[macro_export]
macro_rules! op_FX33 {
    ($adr_i:expr, $opcode:expr, $regs:expr, $mem:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let dec = $regs[regx];
        $mem[$adr_i as usize] = dec % 100;
        $mem[$adr_i as usize + 1] = dec % 10;
        $mem[$adr_i as usize + 2] = dec / 10;
    }};
}

// store V0-X in mem from adr_i
// adr_i += X + 1
#[macro_export]
macro_rules! op_FX55 {
    ($adr_i:expr, $opcode:expr, $regs:expr, $mem:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        for reg in 0..regx + 1 {
            $mem[($adr_i + reg as u16) as usize] = $regs[reg];
        }
        $adr_i += (regx as u16 + 1);
    }};
}

// load from mem from adr_i to adr_i + x into V0-X
// adr_i += X + 1
#[macro_export]
macro_rules! op_FX65 {
    ($adr_i:expr, $opcode:expr, $regs:expr, $mem:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        for reg in 0..regx + 1 {
            $regs[reg] = $mem[($adr_i + reg as u16) as usize];
        }
        $adr_i += (regx + 1) as u16;
    }};
}
