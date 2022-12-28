#[derive(Debug)]

pub struct Cursor {
    line: usize,
    character: usize,
}
#[derive(Debug)]
pub struct Screen {
    pub buffer: [[char; 80]; 12],
    pub cursor: Cursor,
    pub cursor_enabled: bool,
}

pub const SCREEN_ADDR: u8 = 0o341;

impl Screen {
    pub fn new() -> Screen {
        Screen {
            buffer: [[' '; 80]; 12],
            cursor: Cursor {
                line: 0,
                character: 0,
            },
            cursor_enabled: false,
        }
    }

    pub fn get_screen(&self) -> String {
        let mut s = String::new();

        for l in 0..12 {
            for c in 0..80 {
                s.push(self.buffer[l][c]);
            }
            s.push('\n');
        }
        s
    }

    pub fn get_status(&self) -> u8 {
        1 // Write ready is always true
    }

    pub fn write(&mut self, data: u8) {
        self.buffer[self.cursor.line][self.cursor.character] = data as char;
    }

    pub fn set_horizontal(&mut self, data: u8) {
        if data < 80 {
            self.cursor.character = data as usize;
        }
    }

    pub fn set_vertical(&mut self, data: u8) {
        if data < 12 {
            self.cursor.line = data as usize;
        }
    }

    pub fn control_word(&mut self, data: u8) {
        if data & (1 << 1) != 0 {
            // Erase form curser to end of line
            for c in self.cursor.character..80 {
                self.buffer[self.cursor.line][c] = ' ';
            }
        }
        if data & (1 << 2) != 0 {
            // Erase form curser to end of frame
            for c in self.cursor.character..80 {
                for l in self.cursor.line..12 {
                    self.buffer[l][c] = ' ';
                }
            }
        }

        if data & (1 << 4) != 0 {
            // Roll up whole page
            for c in 0..80 {
                for l in 0..11 {
                    self.buffer[l][c] = self.buffer[l + 1][c];
                }
            }

            for c in 0..80 {
                self.buffer[11][c] = ' ';
            }
        }
        self.cursor_enabled = data & (1 << 4) != 0
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}
