use log::info;

#[derive(Debug, Clone)]
pub struct Keyboard {
    display_pressed: bool,
    keyboard_pressed: bool,
    key_buf: u8,
    key_ready: bool,
}

pub const KEYBOARD_ADDR: u8 = 0o341;

fn convert_key(key: String) -> Option<u8> {
    if key.len() == 1 {
        let key: Vec<char> = key.chars().collect();
        if key[0].is_ascii() {
            return Some(key[0] as u8);
        }
    }

    return match key.as_str() {
        "Enter" => Some(13),
        "Cancel" => Some(24),
        "Backspace" => Some(8),
        "Delete" => Some(127),
        _ => None,
    };
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            key_buf: 0,
            key_ready: false,
            keyboard_pressed: false,
            display_pressed: false,
        }
    }

    pub fn keydown(&mut self, key: String) {
        if let Some(key_code) = convert_key(key.clone()) {
            self.key_buf = key_code;
            self.key_ready = true;
            info!("Got key: {}", key);
        }

        if key == "Tab" {
            self.display_pressed = true;
        }

        if key == "Keyboard" {
            self.keyboard_pressed = true;
        }
    }

    pub fn keyup(&mut self, key: String) {
        if let Some(key_code) = convert_key(key.clone()) {
            if key_code == self.key_buf {
                self.key_ready = false;
                info!("Key release");
            }
        }

        if key == "Tab" {
            self.display_pressed = false;
        }

        if key == "Keyboard" {
            self.keyboard_pressed = false;
        }

    }

    pub fn get_status(&self) -> u8 {
        let mut status :u8 = 0;
        if self.key_ready {
            status |= (1<<1);
        }

        if self.keyboard_pressed {
            status |= (1<<2);
        }

        if self.display_pressed {
            status |= (1<<3);
        }
    
        status
    }

    pub fn strobe(&mut self) {
        self.key_ready = false;
    }

    pub fn clock(&self) {}

    pub fn get_data(&mut self) -> u8 {
        self.key_buf
    }

    pub fn write_data(&mut self, data: u8) {}
}
