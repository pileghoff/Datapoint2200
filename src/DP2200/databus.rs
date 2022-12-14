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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Databus {
    pub selected_addr: u8,
    pub selected_mode: DatabusMode,
    pub screen: Screen,
    pub keyboard: Keyboard,
    pub cassette: Cassette,
}

impl Databus {
    fn read_status(&mut self) -> u8 {
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

            return status;
        }
        0
    }

    fn read_data(&mut self) -> u8 {
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
            info!("Get data: {}", data);
            return data;
        }
        0
    }

    pub fn read_bus(&mut self) -> u8 {
        self.read_data() | self.read_status()
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

        self.read_status();
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

    pub fn set_addr(&mut self, addr: u8) {
        self.selected_addr = addr;
        self.selected_mode = DatabusMode::Status;
    }

    pub fn set_mode(&mut self, mode: DatabusMode) {
        self.selected_mode = mode;
    }

    pub fn execute_command(&mut self, inst: Instruction, data: u8) {
        match inst.instruction_type {
            InstructionType::Adr => {
                self.set_addr(data);
            }
            InstructionType::Write => {
                self.write_data(data);
            }
            InstructionType::Com1 => {
                if self.selected_addr == SCREEN_ADDR {
                    self.screen.control_word(data);
                }
            }
            InstructionType::Com2 => {
                if self.selected_addr == SCREEN_ADDR {
                    self.screen.set_horizontal(data);
                }
            }
            InstructionType::Com3 => {
                if self.selected_addr == SCREEN_ADDR {
                    self.screen.set_vertical(data);
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

    pub fn update(&mut self) {
        self.read_data();
        self.read_status();
    }
}
