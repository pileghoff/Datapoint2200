use std::{
    sync::mpsc::{Receiver, Sender, TryRecvError},
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use crate::instruction::{Instruction, InstructionType};

#[derive(Debug, PartialEq, Eq)]
pub enum DatabusMode {
    Data,
    Status,
}

pub type Dataline = Arc<RwLock<u8>>;

#[derive(Debug)]
pub struct Databus {
    pub selected_addr: u8,
    pub selected_mode: DatabusMode,
    pub clock: Receiver<u8>,
    pub command: Receiver<Instruction>,
    pub data_input: Dataline,
    pub data_output: Dataline,
}

impl Databus {
    pub fn run(mut self) -> JoinHandle<Databus> {
        thread::spawn(move || {
            loop {
                match self.clock.try_recv() {
                    Ok(_) => {
                        // 153.6kHz Clock
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                };

                match self.command.try_recv() {
                    Ok(inst) => match inst.instruction_type {
                        InstructionType::Adr => {
                            println!("Set addr");
                            if let Ok(val) = self.data_input.read() {
                                self.selected_addr = *val;
                                self.selected_mode = DatabusMode::Status;
                                println!("Set addr {}", *val);
                            }
                        }
                        InstructionType::Status => {
                            self.selected_mode = DatabusMode::Status;
                        }
                        InstructionType::Data => {
                            self.selected_mode = DatabusMode::Data;
                        }
                        InstructionType::Write => todo!(),
                        InstructionType::Com1 => todo!(),
                        InstructionType::Com2 => todo!(),
                        InstructionType::Com3 => todo!(),
                        InstructionType::Com4 => todo!(),
                        InstructionType::Beep => todo!(),
                        InstructionType::Click => todo!(),
                        InstructionType::Deck1 => todo!(),
                        InstructionType::Deck2 => todo!(),
                        InstructionType::Rbk => todo!(),
                        InstructionType::Wbk => todo!(),
                        InstructionType::Bsp => todo!(),
                        InstructionType::Sf => todo!(),
                        InstructionType::Sb => todo!(),
                        InstructionType::Rewind => todo!(),
                        InstructionType::Tstop => todo!(),
                        InstructionType::Halt => {
                            break;
                        }
                        _ => {}
                    },
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                };
            }

            return self;
        })
    }
}
