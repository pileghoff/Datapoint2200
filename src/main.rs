#![allow(dead_code)]

pub mod assembler;
pub mod cpu;
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
use std::io;

fn main() -> Result<()> {
    let program = assemble(vec![
        //                                     Number of times instruction is run
        "LoadImm A, 10",     // 10                       =  1
        "loop: SubImm 1",    // 9, 8,7,6,5,4,3,2,1,0,-1  = 11
        "JumpIfNot 0, loop", //9,8,7,6,5,4,3,2,1,0,-1    = 11
        "Halt",              // -1                       =  1
    ]); //                                                 24 total

    let mut cpu = Cpu::new(program);
    let disassembler = Disassembler::new(cpu.clone());

    while !cpu.halted {
        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
        let output = disassembler.get_lines(cpu.program_counter, 5, 2);
        print!("{}", output);
        event::read();
        fetch_instruction(&mut cpu);
        execute_instruction(&mut cpu);
    }

    Ok(())
}
