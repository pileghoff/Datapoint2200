use std::{
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    sync::{Arc, RwLock},
};

use crate::DP2200::screen::Screen;
use crate::DP2200::{
    instruction::{Instruction, InstructionType},
    screen::SCREEN_ADDR,
};

#[derive(Debug, PartialEq, Eq)]
pub enum DatabusMode {
    Data,
    Status,
}

#[derive(Debug)]
pub struct Dataline {
    writer: Arc<RwLock<u8>>,
    reader: Arc<RwLock<u8>>,
    command_sender: Sender<Instruction>,
    command_receiver: Receiver<Instruction>,
}

impl Dataline {
    pub fn generate_pair() -> (Dataline, Dataline) {
        let left = Arc::new(RwLock::new(0));
        let right = Arc::new(RwLock::new(0));
        let command_left = channel();
        let command_right = channel();

        (
            Dataline {
                writer: left.clone(),
                reader: right.clone(),
                command_sender: command_left.0,
                command_receiver: command_right.1,
            },
            Dataline {
                writer: right,
                reader: left,
                command_sender: command_right.0,
                command_receiver: command_left.1,
            },
        )
    }

    pub fn read(&self) -> u8 {
        *self.reader.read().unwrap()
    }

    pub fn write(&mut self, val: u8) {
        *self.writer.write().unwrap() = val;
    }

    pub fn send_command(&self, inst: Instruction) {
        self.command_sender.send(inst).unwrap();
    }

    pub fn get_command(&self) -> Result<Instruction, TryRecvError> {
        self.command_receiver.try_recv()
    }
}

#[derive(Debug)]
pub struct Databus {
    pub selected_addr: u8,
    pub selected_mode: DatabusMode,
    pub clock: Receiver<u8>,
    pub dataline: Dataline,
    pub screen: Screen,
}

impl Databus {
    pub fn run(&mut self) {
        match self.clock.try_recv() {
            Ok(_) => {}
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        };

        match self.dataline.get_command() {
            Ok(inst) => match inst.instruction_type {
                InstructionType::Adr => {
                    println!("Set addr");
                    self.selected_addr = self.dataline.read();
                    self.selected_mode = DatabusMode::Status;
                }
                InstructionType::Status => {
                    self.selected_mode = DatabusMode::Status;
                }
                InstructionType::Data => {
                    self.selected_mode = DatabusMode::Data;
                }
                InstructionType::Write => {
                    if self.selected_addr == SCREEN_ADDR {
                        self.screen.write(self.dataline.read());
                    }
                }
                InstructionType::Com1 => {
                    if self.selected_addr == SCREEN_ADDR {
                        self.screen.control_word(self.dataline.read());
                    }
                }
                InstructionType::Com2 => {
                    if self.selected_addr == SCREEN_ADDR {
                        self.screen.set_horizontal(self.dataline.read());
                    }
                }
                InstructionType::Com3 => {
                    if self.selected_addr == SCREEN_ADDR {
                        self.screen.set_vertical(self.dataline.read());
                    }
                }
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
                InstructionType::Halt => {}
                _ => {}
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        };
    }
}
