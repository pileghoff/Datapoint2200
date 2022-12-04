#![allow(dead_code)]

pub mod assembler;
pub mod clock;
pub mod cpu;
pub mod databus;
pub mod datapoint;
pub mod disassembler;
pub mod instruction;
use assembler::assemble;
use cpu::{execute_instruction, fetch_instruction, Cpu};
use crossterm::{
    cursor, event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal, ExecutableCommand, Result,
};

use disassembler::Disassembler;
use instruction::{InstructionType, FLAG_NAME, REG_NAME};
use std::io;

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
    // let program = assemble(vec![
    //     "LoadImm A, 10",
    //     "loop: SubImm 1",
    //     "JumpIfNot 0, loop",
    //     "Halt",
    // ]);
    let program = assemble(vec![
        "CompImm 1",      // Carry flag gets set if A is 0 (First time)
        "JumpIf Cf, run", // Jump if A was 0
        "Halt",           // Only halt, only get here if the interrupts actually works
        "run: LoadImm A, 1",
        "EnableIntr",
        "Nop",
        "Jump run",
    ]);

    let mut cpu = Cpu::new(program.clone());
    let disassembler = Disassembler::new(Cpu::new(program));

    while !cpu.halted {
        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
        let output = disassembler.get_lines(cpu.program_counter, 5, 2);
        render_cpu_regs(&cpu);
        render_cpu_flags(&cpu);
        render_disassembler_out(output);

        event::read();
        fetch_instruction(&mut cpu);
        execute_instruction(&mut cpu);
    }

    Ok(())
}
