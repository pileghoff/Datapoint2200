// From datasheet
// Speed: 2.8ms pr byte
// Rewind 4230 bytes pr sec -> 236us pr byte
// 153.6kHz clock -> 6.5us cycle

// Normal:
// 431 cycles pr byte

// Rewind:
// 36 cycles pr byte

use std::collections::VecDeque;

use log::{error, info};

use log::{trace, warn};

pub const CASSETTE_ADDR: u8 = 0o360;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CassetteData {
    Data(u8),
    Gap,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementSpeed {
    None,
    Regular,
    Rewind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDirection {
    Forward,
    Backwards,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckId {
    Deck1,
    Deck2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CassetteDeck {
    data: Vec<CassetteData>,
    pub speed: MovementSpeed,
    pub direction: MovementDirection,
    pub head_pos: usize,
    cycle_count: usize,
    data_buf: Option<u8>,
    gap_detected: bool,
    ignore_gap: bool,
    stop_on_gap: bool,
}

fn parse_tap(data: &mut Vec<u8>) -> Vec<CassetteData> {
    let mut cassette_data = Vec::new();
    let mut gap = [CassetteData::Gap].repeat(10);
    cassette_data.append(&mut gap);
    while !data.is_empty() {
        let sec_len_bytes: [u8; 4] = data[0..4].try_into().unwrap();
        let sec_len = u32::from_le_bytes(sec_len_bytes) as usize;
        data.drain(0..4);

        let d = data.drain(0..sec_len).map(CassetteData::Data);
        cassette_data.append(&mut d.collect());
        let mut gap = [CassetteData::Gap].repeat(10);
        cassette_data.append(&mut gap);

        data.drain(0..4);
    }

    cassette_data
}

impl CassetteDeck {
    pub fn new(tap_file: Vec<u8>) -> CassetteDeck {
        let mut tap_file = tap_file;
        let data = parse_tap(&mut tap_file);

        CassetteDeck {
            data,
            speed: MovementSpeed::None,
            direction: MovementDirection::Forward,
            head_pos: 0,
            cycle_count: 0,
            data_buf: None,
            gap_detected: false,
            ignore_gap: false,
            stop_on_gap: false,
        }
    }

    fn read_data(&mut self) {
        match self.data.get(self.head_pos) {
            Some(CassetteData::Data(data)) => {
                self.data_buf = Some(*data);
                self.gap_detected = false;
                self.ignore_gap = false;
            }
            Some(CassetteData::Gap) => {
                if !self.ignore_gap {
                    self.data_buf = None;
                    self.gap_detected = true;

                    if self.stop_on_gap {
                        self.speed = MovementSpeed::None;
                    }
                }
            }
            None => {}
        }
    }

    pub fn update_head(&mut self) {
        match self.direction {
            MovementDirection::Forward => {
                if self.head_pos < self.data.len() - 1 {
                    self.head_pos += 1;
                    self.read_data();
                } else {
                    self.speed = MovementSpeed::None;
                }
            }
            MovementDirection::Backwards => {
                if self.head_pos > 0 {
                    self.head_pos -= 1;
                    self.read_data();
                } else {
                    self.speed = MovementSpeed::None;
                }
            }
        }
    }

    pub fn clock(&mut self) {
        // 153.6kHz clock -> 6.5us cycle
        // 2.8ms pr byte -> 431 cycles pr byte
        self.cycle_count += 1;

        let cycle_goal = match self.speed {
            MovementSpeed::None => None,
            //MovementSpeed::Regular => Some(431),
            MovementSpeed::Regular => Some(50),
            MovementSpeed::Rewind => Some(36),
        };

        if let Some(cycle_goal) = cycle_goal {
            if self.cycle_count >= cycle_goal {
                self.cycle_count = 0;
                self.update_head();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cassette {
    deck1: CassetteDeck,
    deck2: CassetteDeck,
    pub selected_deck: DeckId,
    data_buffer: VecDeque<u8>,
}

impl Cassette {
    pub fn new() -> Cassette {
        Cassette {
            deck1: CassetteDeck::new(Vec::new()),
            deck2: CassetteDeck::new(Vec::new()),
            selected_deck: DeckId::Deck1,
            data_buffer: VecDeque::new(),
        }
    }

    pub fn clock(&mut self) {
        let mut gap_detected = false;
        let deck = self.get_selected_deck();
        deck.clock();

        if let Some(read_data) = deck.data_buf.take() {
            self.data_buffer.push_front(read_data);
            self.data_buffer.truncate(2);
        }
    }

    pub fn get_selected_deck(&mut self) -> &mut CassetteDeck {
        match self.selected_deck {
            DeckId::Deck1 => &mut self.deck1,
            DeckId::Deck2 => &mut self.deck2,
        }
    }

    pub fn get_status(&mut self) -> u8 {
        let mut status = 0;
        if !self.data_buffer.is_empty() {
            status |= 1 << 2;
        }

        let deck = self.get_selected_deck();
        if deck.speed == MovementSpeed::None && !deck.data.is_empty() {
            status |= 1 << 0;
        }

        if deck.head_pos == 0 || deck.head_pos == deck.data.len() - 1 {
            status |= 1 << 1;
        }

        // Write ready not implemented

        if deck.gap_detected {
            status |= 1 << 4;
        }

        if !deck.data.is_empty() {
            status |= 1 << 6;
        }

        status
    }

    pub fn strobe(&mut self) {
        self.data_buffer.pop_back();
    }

    pub fn ex_tstop(&mut self) {
        let deck = self.get_selected_deck();
        deck.speed = MovementSpeed::None;
        deck.data_buf = None;
        self.data_buffer.clear();
    }

    pub fn ex_deck1(&mut self) {
        self.selected_deck = DeckId::Deck1;
    }

    pub fn ex_deck2(&mut self) {
        self.selected_deck = DeckId::Deck2;
    }

    pub fn ex_rbk(&mut self) {
        let deck = self.get_selected_deck();
        deck.direction = MovementDirection::Forward;
        deck.speed = MovementSpeed::Regular;
        if let Some(CassetteData::Gap) = deck.data.get(deck.head_pos) {
            deck.ignore_gap = true;
        }

        deck.stop_on_gap = true;
    }

    pub fn ex_bsp(&mut self) {
        let deck = self.get_selected_deck();
        deck.direction = MovementDirection::Backwards;
        deck.speed = MovementSpeed::Regular;
        if let Some(CassetteData::Gap) = deck.data.get(deck.head_pos) {
            deck.ignore_gap = true;
            deck.gap_detected = false;
        }
    }

    pub fn ex_sf(&mut self) {
        let deck = self.get_selected_deck();
        deck.direction = MovementDirection::Forward;
        deck.speed = MovementSpeed::Regular;
        if let Some(CassetteData::Gap) = deck.data.get(deck.head_pos) {
            deck.ignore_gap = true;
        }

        deck.stop_on_gap = false;
    }

    pub fn ex_sb(&mut self) {
        let deck = self.get_selected_deck();
        deck.direction = MovementDirection::Backwards;
        deck.speed = MovementSpeed::Regular;
        if let Some(CassetteData::Gap) = deck.data.get(deck.head_pos) {
            deck.ignore_gap = true;
        }

        deck.stop_on_gap = false;
    }

    pub fn get_data(&mut self) -> u8 {
        let data = self.data_buffer.back().copied();
        if let Some(data) = data {
            return data;
        }
        0
    }

    pub fn write_data(&mut self, data: u8) {}

    pub fn load(&mut self, deck: DeckId, tap_file: Vec<u8>) {
        match deck {
            DeckId::Deck1 => self.deck1 = CassetteDeck::new(tap_file),
            DeckId::Deck2 => self.deck2 = CassetteDeck::new(tap_file),
        }
    }

    pub fn get_first_sector(&mut self) -> Vec<u8> {
        let mut data_out = Vec::new();
        self.ex_deck1();
        self.ex_tstop();
        self.deck1.head_pos = 0;
        self.ex_rbk();
        while !self.deck1.gap_detected {
            self.clock();
            if !self.data_buffer.is_empty() {
                data_out.push(self.get_data());
                self.strobe();
            }
        }

        data_out
    }
}

#[cfg(test)]
mod tests {
    use crate::DP2200::{cassette::*, datapoint::Datapoint};
    fn init_logger() {
        let _ = env_logger::builder()
            // Include all events in tests
            .filter_level(log::LevelFilter::max())
            // Ensure events are captured by `cargo test`
            .is_test(true)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();
    }

    #[test]
    fn test_read_block() {
        init_logger();
        let mut cassettes = Cassette::new();
        let tap_file = vec![
            2, 0, 0, 0, 0xbe, 0xef, 2, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5, 5, 0, 0, 0,
        ];
        cassettes.load(DeckId::Deck1, tap_file);
        cassettes.ex_deck1();
        cassettes.ex_rbk();
        let mut data_out = Vec::new();
        while cassettes.get_status() & (1 << 4) == 0 {
            cassettes.clock();
            if cassettes.get_status() & (1 << 2) != 0 {
                data_out.push(cassettes.get_data());
                cassettes.strobe();
            }
        }

        assert_eq!(data_out, vec![0xbe, 0xef]);
        assert_eq!(cassettes.deck1.speed, MovementSpeed::None);
    }

    #[test]
    fn test_read_backwards() {
        init_logger();
        let mut cassettes = Cassette::new();
        let tap_file = vec![
            2, 0, 0, 0, 0xbe, 0xef, 2, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5, 5, 0, 0, 0,
        ];
        cassettes.load(DeckId::Deck1, tap_file);
        cassettes.ex_deck1();
        cassettes.ex_rbk();
        while cassettes.get_status() & (1 << 4) == 0 {
            cassettes.clock();
            cassettes.strobe();
        }

        let mut data_out = Vec::new();
        cassettes.ex_bsp();
        while cassettes.get_status() & (1 << 4) == 0 {
            cassettes.clock();
            if cassettes.get_status() & (1 << 2) != 0 {
                data_out.push(cassettes.get_data());
                cassettes.strobe();
            }
        }

        assert_eq!(data_out, vec![0xef, 0xbe]);
        assert_eq!(cassettes.deck1.speed, MovementSpeed::None);
    }

    #[test]
    fn test_read_sector() {
        let program = include_bytes!("../../test_software/dosAbootVer2.tap").to_vec();

        let mut machine = Datapoint::build(&Vec::new(), 1.0);
        machine.load_cassette(program);
    }

    #[test]
    fn test_read_sector_2() {
        let mut cassettes = Cassette::new();
        let tap_file = include_bytes!("../../test_software/dosAbootVer2.tap").to_vec();
        cassettes.load(DeckId::Deck1, tap_file);
        let sector = cassettes.get_first_sector();
        assert_eq!(sector.len(), 512);
    }

    #[test]
    fn test_read_from_cpu() {
        init_logger();
        let tap_file = vec![
            2, 0, 0, 0, 0xbe, 0xef, 2, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5, 5, 0, 0, 0,
        ];

        let program = vec![
            "LoadImm A, 0xf0",
            "Adr",
            "Rbk",
            "dat1: Input",
            "AndImm 4",
            "JumpIf Zf, dat1",
            "Data",
            "Input",
            "Load B, A",
            "Status",
            "dat2: Input",
            "AndImm 4",
            "JumpIf Zf, dat2",
            "Data",
            "Input",
            "Load C, A",
            "Status",
            "dat3: Input",
            "AndImm 16",
            "JumpIf Zf, dat3",
            "Halt",
        ];

        let mut machine = Datapoint::new(program, 1.0);
        machine.databus.cassette.deck1 = CassetteDeck::new(tap_file);

        machine.run();
        assert_eq!(machine.cpu.alpha_registers[1], 0xbe);
        assert_eq!(machine.cpu.alpha_registers[2], 0xef);
    }

    #[test]
    fn test_sf_gap_detect() {
        init_logger();
        let mut cassettes = Cassette::new();
        let tap_file = vec![
            2, 0, 0, 0, 0xbe, 0xef, 2, 0, 0, 0, 5, 0, 0, 0, 1, 2, 3, 4, 5, 5, 0, 0, 0,
        ];

        cassettes.load(DeckId::Deck1, tap_file);
        cassettes.ex_deck1();
        cassettes.ex_sf();
        while cassettes.get_status() & (1 << 4) == 0 {
            cassettes.clock();
        }

        info!("Head pos: {}", cassettes.deck1.head_pos)
    }
}
