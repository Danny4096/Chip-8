#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(unused_mut)]
#![allow(dead_code)]

use rand::{thread_rng, Rng};
use sdl2;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs::{metadata, File};
use std::io;
use std::io::prelude::*;
use std::task::Context;
use std::vec;

const CHIP_8_HEIGHT: usize = 32;
const CHIP_8_WIDTH: usize = 64;
const SCREEN_SCALAR: usize = 20;
const ROM_PATH: &str = r"C:\Users\Danyaal\Documents\Rust\chip8\pong.ch8";

fn main() {
    // the chip-8 uses 8 bit data registers and 16 bit address reg., pc, & stack
    type BYTE = u8;
    type WORD = u16;

    let sleep_duration = std::time::Duration::from_millis(15);
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

    let mut delay_timer: BYTE = 0;
    let mut sound_timer: BYTE = 0;

    let mut key_wait: bool = false;
    let mut key_reg: usize = 0;
    let mut keypad: [bool; 16] = [false; 16];

    store_hex_digits(&mut memory);
    rst(&mut memory, &mut address_i, &mut program_counter);
    // the screen is 64*32
    // sprites are drawn using a co-ordinate system
    // sprites have a width of 8 and variable height up to 15
    // SDL2
    // co-ords refer to the top-left pixel of a sprite
    let mut screen_buffer: [[BYTE; CHIP_8_WIDTH]; CHIP_8_HEIGHT] =
        [[0; CHIP_8_WIDTH]; CHIP_8_HEIGHT];
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let audio_device = AudioDriver::new(&sdl_context);
    //audio_device.start_beep();

    let window = video_subsystem
        .window(
            "rust-sdl2 demo: Video",
            (CHIP_8_WIDTH * SCREEN_SCALAR) as u32,
            (CHIP_8_HEIGHT * SCREEN_SCALAR) as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    canvas.set_draw_color(pixels::Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    println!("{:?}", memory);
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        //canvas.clear();
        canvas.present();
        //::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 30));

        keypad = poll_kb(&sdl_context, &mut event_pump);
        if key_wait {
            for keypress in 0..keypad.len() {
                if keypad[keypress] {
                    key_wait = false;
                    registers[key_reg] = keypress as u8;
                    println!("key reg: {}", registers[key_reg]);
                    break;
                }
            }
        } else {
            println!("keypad: {:?}", keypad);
            let opcode: u16 = fetch_opcode(&memory, &mut program_counter);
            println!("opc: {:x}", opcode);
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
                    0x0000 => op_00E0!(screen_buffer),
                    0x000E => op_00EE!(program_counter, opcode, &mut stack),
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
                0xD000 => {
                    let regx = ((opcode & 0x0F00) >> 8) as usize;
                    let regy = ((opcode & 0x00F0) >> 4) as usize;
                    let n = (opcode & 0x000F) as u8;
                    registers[0x0f] = 0;
                    for byte in 0..n {
                        let y = ((registers[regy] as usize + byte as usize) % CHIP_8_HEIGHT);
                        for bit in 0..8 {
                            let x = (registers[regx] as usize + bit) % CHIP_8_WIDTH;
                            let color =
                                ((memory[(address_i + byte as u16) as usize]) >> (7 - bit)) & 1;
                            registers[0x0f] |= color & screen_buffer[y][x];
                            screen_buffer[y][x] ^= color;
                        }
                    }

                    //println!("{:?}", screen_buffer);
                    doodle(&screen_buffer, &mut canvas);
                }
                0xE000 => match opcode & 0x00FF {
                    0x009E => op_EX9E!(program_counter, opcode, keypad),
                    0x00A1 => op_EXA1!(program_counter, opcode, keypad),
                    _ => unreachable!(),
                },
                0xF000 => match opcode & 0x00FF {
                    0x0007 => op_FX07!(delay_timer, opcode, registers),
                    0x000A => op_FX0A!(key_wait, key_reg, opcode),
                    0x0015 => op_FX15!(delay_timer, opcode, registers),
                    0x0018 => op_FX18!(sound_timer, opcode, registers),
                    0x001E => op_FX1E!(address_i, opcode, registers),
                    0x0029 => op_FX29!(address_i, opcode, registers),
                    0x0033 => op_FX33!(address_i, opcode, registers, memory),
                    0x0055 => op_FX55!(address_i, opcode, registers, memory),
                    0x0065 => op_FX65!(address_i, opcode, registers, memory),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
            println!("pc: {:x}", program_counter);

            if delay_timer > 0 {
                delay_timer -= 1;
            }
            if sound_timer > 0 {
                audio_device.start_beep();
                sound_timer -= 1;
            } else {
                audio_device.stop_beep();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}

fn doodle(vram: &[[u8; CHIP_8_WIDTH]; CHIP_8_HEIGHT], canvas: &mut Canvas<Window>) {
    for (y, row) in vram.iter().enumerate() {
        for (x, &col) in row.iter().enumerate() {
            let x = x * SCREEN_SCALAR;
            let y = y * SCREEN_SCALAR;
            canvas.set_draw_color(match col {
                1 => pixels::Color::RGB(255, 255, 255),
                0 => pixels::Color::RGB(0, 0, 0),
                _ => pixels::Color::RGB(255, 0, 0),
            });

            let _ = canvas.fill_rect(sdl2::rect::Rect::new(
                x as i32,
                y as i32,
                SCREEN_SCALAR as u32,
                SCREEN_SCALAR as u32,
            ));
        }
    }
    canvas.present();
    //println!("written to screen?");
}

fn poll_kb(context: &sdl2::Sdl, events: &mut sdl2::EventPump) -> [bool; 16] {
    for event in events.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(1),
            _ => {}
        }
    }

    let keys_pressed: Vec<Keycode> = events
        .keyboard_state()
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect();

    let mut keypad: [bool; 16] = [false; 16];

    // itereate over keys scanned from kb
    for key in keys_pressed {
        // assign index in the keypad array based on the output of the match
        /*
        keypad layout
        1   2   3   C
        4   5   6   D
        7   8   9   E
        A   0   B   F
        */
        let index = match key {
            Keycode::U => Some(0x1),
            Keycode::I => Some(0x2),
            Keycode::O => Some(0x3),
            Keycode::P => Some(0xc),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xd),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xe),
            Keycode::Z => Some(0xa),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xb),
            Keycode::V => Some(0xf),
            _ => None,
        };

        if let Some(i) = index {
            keypad[i] = true;
        }
    }
    keypad
}

fn rst(mem: &mut [u8; 0xFFF], adr: &mut u16, pc: &mut u16) {
    *adr = 0;
    // the CHIP-8 interpreter itself occupies the first 512 bytes of the memory space
    *pc = 0x200;

    // load chip-8 rom from file to mem array
    let mut f: File = File::open(ROM_PATH).expect("could not read file");
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

pub struct AudioDriver {
    device: AudioDevice<SquareWave>,
}

impl AudioDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // Show obtained AudioSpec
                println!("{:?}", spec);

                // initialize the audio callback
                SquareWave {
                    phase_inc: 240.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                }
            })
            .unwrap();

        AudioDriver { device: device }
    }

    pub fn start_beep(&self) {
        self.device.resume();
    }
    pub fn stop_beep(&self) {
        self.device.pause();
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn op_00E0() {}
fn op_00EE() {}

#[macro_export]
macro_rules! op_00EE {
    ($pc:expr, $opcode:expr, $stack:expr) => {{
        $pc = $stack.pop().unwrap()
    }};
}

#[macro_export]
macro_rules! op_00E0 {
    ($vram:expr) => {
        $vram = [[0; CHIP_8_WIDTH]; CHIP_8_HEIGHT]
    };
}

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
        let c = $pc;
        $stack.push(c);
        $pc = $opcode & 0x0FFF
    }};
}

// skip if Vx == NN
#[macro_export]
macro_rules! op_3XNN {
    ($pc:expr, $opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let n = ($opcode & 0x00FF) as u8;
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
        let n = ($opcode & 0x00FF) as u8;
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
        $regs[regx] = $regs[regx].wrapping_add(n as u8);
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
        let (result, carry) = $regs[regx].overflowing_add($regs[regy]);
        $regs[regx] = result;
        $regs[0xF] = if carry { 1 } else { 0 };
    }};
}

// Vx -= Vy; VF = 1 if overflow
#[macro_export]
macro_rules! op_8XY5 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        //let (result, borrow) = $regs[regx].overflowing_sub($regs[regy]);
        let valx = $regs[regx];
        let valy = $regs[regy];
        $regs[regx] = $regs[regx].wrapping_sub($regs[regy]);
        $regs[0x0f] = if valx > valy { 1 } else { 0 };

        //$regs[regx] = result;
        //$regs[0xF] = if borrow { 1 } else { 0 };
    }};
}

// store lsb and bit shift right
#[macro_export]
macro_rules! op_8XY6 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        let val = $regs[regy];
        $regs[regx] = val >> 1;
        $regs[0xF] = (val & 1) as u8;
    }};
}

// Vx = Vy - Vx
#[macro_export]
macro_rules! op_8XY7 {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        let (new_vx, borrow) = $regs[regy].overflowing_sub($regs[regx]);
        $regs[regx] = new_vx;
        $regs[0xF] = if borrow { 0 } else { 1 };
    }};
}

// store msb in VF and bit shift Vy left
#[macro_export]
macro_rules! op_8XYE {
    ($opcode:expr, $regs:expr) => {{
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 4) as usize;
        let val = $regs[regy];
        $regs[regx] = val << 1;
        $regs[0xF] = (val >> 7) as u8;
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
    ($adr_i:expr, $opcode:expr, $regs:expr, $vram:expr, $mem:expr, $canv:expr) => {{
        // decode x and y
        let regx = (($opcode & 0x0F00) >> 8) as usize;
        let regy = (($opcode & 0x00F0) >> 8) as usize;
        let n = (($opcode & 0x000F) >> 8) as u8;
        let flip = false;
        for byte in 0..n {
            // top corner + current row of sprite (modded w screen height for wrapping)
            let y = (($regs[regy] + byte) % 32) as usize;
            let sprite_data = $mem[($adr_i + byte as u16) as usize];
            for bit in 0..8 {
                let x = (($regs[regy] + bit) % 64) as usize;
                let pixel = (sprite_data >> (7 - bit)) & 1;
                flip |= $vram[y][x];
                $vram[x][y] ^= pixel;
            }
        }
        if flip {
            $regs[0xF] = 1;
        } else {
            $regs[0xF] = 0;
        }
        doodle($vram, &mut $canv);
    }};
}

// input
fn op_EX9E() {}

#[macro_export]
macro_rules! op_EX9E {
    ($pc:expr, $opcode:expr, $keypad:expr) => {
        if ($keypad[(($opcode & 0x0F00) >> 8) as usize]) {
            $pc += 2;
            println!("got key press: {}", (($opcode & 0x0F00) >> 8))
        }
    };
}

fn op_EXA1() {}

#[macro_export]
macro_rules! op_EXA1 {
    ($pc:expr, $opcode:expr, $keypad:expr) => {
        if (!$keypad[(($opcode & 0x0F00) >> 8) as usize]) {
            $pc += 2;
        }
    };
}

// store delay timer val in Vx
#[macro_export]
macro_rules! op_FX07 {
    ($delay:expr, $opcode:expr, $regs:expr) => {
        $regs[(($opcode & 0x0F00) >> 8) as usize] = $delay
    };
}

// Wait for a keypress and store the result in Vx
#[macro_export]
macro_rules! op_FX0A {
    ($key_wait:expr, $key_reg:expr, $opcode:expr) => {{
        $key_wait = true;
        $key_reg = (($opcode & 0x0F00) >> 8) as usize;
    }};
}

//delay timer
#[macro_export]
macro_rules! op_FX15 {
    ($delay:expr, $opcode:expr, $regs:expr) => {
        $delay = $regs[(($opcode & 0x0F00) >> 8) as usize]
    };
}

// sound timer
#[macro_export]
macro_rules! op_FX18 {
    ($sound:expr, $opcode:expr, $regs:expr) => {
        $sound = $regs[(($opcode & 0x0F00) >> 8) as usize]
    };
}

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
        $mem[$adr_i as usize] = dec / 100;
        $mem[$adr_i as usize + 1] = (dec % 100) / 10;
        $mem[$adr_i as usize + 2] = dec % 10;
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
