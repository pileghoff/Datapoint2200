#![allow(dead_code)]

pub mod assembler;
pub mod clock;
pub mod cpu;
pub mod databus;
pub mod datapoint;
pub mod disassembler;
pub mod instruction;
pub mod screen;

use assembler::assemble;
use cpu::Cpu;
use crossterm::{cursor, ExecutableCommand, Result};

use datapoint::Datapoint;
use instruction::{FLAG_NAME, REG_NAME};
use std::{env, fs, io};

fn render_disassembler_out(lines: Vec<String>) {
    io::stdout().execute(cursor::MoveTo(0, 0)).unwrap();
    println!("--- Disassembly ----");
    for (i, l) in lines.iter().enumerate() {
        io::stdout()
            .execute(cursor::MoveTo(0, 1 + i as u16))
            .unwrap();
        println!("{}", l);
    }
}

fn render_cpu_regs(cpu: &Cpu) {
    io::stdout().execute(cursor::MoveTo(30, 0)).unwrap();
    println!("--- Alpha regs ----");
    for (i, r) in cpu.alpha_registers.iter().enumerate() {
        io::stdout()
            .execute(cursor::MoveTo(30, (i + 1) as u16))
            .unwrap();
        println!("{}: {}", REG_NAME[i], r);
    }
}

fn render_cpu_flags(cpu: &Cpu) {
    io::stdout().execute(cursor::MoveTo(60, 0)).unwrap();
    println!("--- Alpha flags ----");
    for (i, r) in cpu.alpha_flipflops.iter().enumerate() {
        io::stdout()
            .execute(cursor::MoveTo(60, (i + 1) as u16))
            .unwrap();
        println!("{}: {}", FLAG_NAME[i], r);
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "./test_software/banner.asm"
    };

    let contents = fs::read_to_string(file_path).unwrap();

    let mut machine = Datapoint::new(contents.lines().collect(), 1.0);
    machine.run();
    let db = machine.databus.take().unwrap();
    for line in db.screen.buffer {
        for c in line {
            print!("{c}");
        }
        println!("");
    }
    Ok(())
}
