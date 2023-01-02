use std::{
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    sync::{Arc, RwLock},
};

use log::info;

use crate::DP2200::{cassette::Cassette, screen::Screen};
use crate::DP2200::{
    cassette::CASSETTE_ADDR,
    instruction::{Instruction, InstructionType},
    keyboard::KEYBOARD_ADDR,
    screen::SCREEN_ADDR,
};

use super::keyboard::Keyboard;

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
    strobe_sender: Sender<u8>,
    strobe_receiver: Receiver<u8>,
}

impl Dataline {
    pub fn generate_pair() -> (Dataline, Dataline) {
        let left = Arc::new(RwLock::new(0));
        let right = Arc::new(RwLock::new(0));
        let command_left = channel();
        let command_right = channel();

        let strobe_left = channel();
        let strobe_right = channel();

        (
            Dataline {
                writer: left.clone(),
                reader: right.clone(),
                command_sender: command_left.0,
                command_receiver: command_right.1,
                strobe_sender: strobe_left.0,
                strobe_receiver: strobe_right.1,
            },
            Dataline {
                writer: right,
                reader: left,
                command_sender: command_right.0,
                command_receiver: command_left.1,
                strobe_sender: strobe_right.0,
                strobe_receiver: strobe_left.1,
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

    pub fn send_strobe(&self) {
        self.strobe_sender.send(0).unwrap();
    }

    pub fn get_strobe(&self) -> bool {
        self.strobe_receiver.try_recv().is_ok()
    }
}

#[derive(Debug)]
pub struct Databus {
    pub selected_addr: u8,
    pub selected_mode: DatabusMode,
    pub clock: Receiver<u8>,
    pub dataline: Dataline,
    pub screen: Screen,
    pub keyboard: Keyboard,
    pub cassette: Cassette,
}

impl Databus {
    fn update_status(&mut self) {
        if self.selected_mode == DatabusMode::Status {
            let mut status = 0;
            if self.selected_addr == CASSETTE_ADDR {
                status |= self.cassette.get_status();
            }

            if self.selected_addr == SCREEN_ADDR {
                status |= self.screen.get_status();
            }

            if self.selected_addr == KEYBOARD_ADDR {
                status |= self.keyboard.get_status();
            }

            self.dataline.write(status);
        }
    }

    fn update_data(&mut self) {
        if self.selected_mode == DatabusMode::Data {
            let mut data = 0;
            if self.selected_addr == CASSETTE_ADDR {
                data |= self.cassette.get_data();
            }

            if self.selected_addr == SCREEN_ADDR {
                data |= self.screen.get_data();
            }

            if self.selected_addr == KEYBOARD_ADDR {
                data |= self.keyboard.get_data();
            }

            self.dataline.write(data);
        }
    }

    pub fn write_data(&mut self, data: u8) {
        if self.selected_addr == CASSETTE_ADDR {
            self.cassette.write_data(data)
        }

        if self.selected_addr == SCREEN_ADDR {
            self.screen.write_data(data);
        }

        if self.selected_addr == KEYBOARD_ADDR {
            self.keyboard.write_data(data);
        }
    }

    pub fn clock(&mut self) {
        if self.selected_addr == CASSETTE_ADDR {
            self.cassette.clock();
        }

        if self.selected_addr == SCREEN_ADDR {
            self.screen.clock();
        }

        if self.selected_addr == KEYBOARD_ADDR {
            self.keyboard.clock();
        }

        self.update_status();
    }

    pub fn strobe(&mut self) {
        if self.selected_mode == DatabusMode::Data {
            if self.selected_addr == KEYBOARD_ADDR {
                self.keyboard.strobe();
            }
            if self.selected_addr == SCREEN_ADDR {
                self.screen.strobe();
            }

            if self.selected_addr == CASSETTE_ADDR {
                self.cassette.strobe();
            }
        }
    }

    pub fn execute_command(&mut self, inst: Instruction) {
        match inst.instruction_type {
            InstructionType::Adr => {
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
                self.write_data(self.dataline.read());
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
            InstructionType::Deck1 => self.cassette.ex_deck1(),
            InstructionType::Deck2 => self.cassette.ex_deck2(),
            InstructionType::Rbk => self.cassette.ex_rbk(),
            InstructionType::Wbk => todo!(),
            InstructionType::Bsp => self.cassette.ex_bsp(),
            InstructionType::Sf => self.cassette.ex_sf(),
            InstructionType::Sb => self.cassette.ex_sb(),
            InstructionType::Rewind => todo!(),
            InstructionType::Tstop => self.cassette.ex_tstop(),
            InstructionType::Halt => {}
            _ => {}
        }
    }

    pub fn run(&mut self) {
        match self.clock.try_recv() {
            Ok(_) => {
                self.clock();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        };

        if self.dataline.get_strobe() {
            self.strobe();
        }

        let command = self.dataline.get_command();
        match command {
            Ok(inst) => self.execute_command(inst),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        };

        self.update_data();
        self.update_status();
    }
}
