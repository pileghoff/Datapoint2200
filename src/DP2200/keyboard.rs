use log::{info, trace};

#[derive(Debug, Clone)]
pub struct Keyboard {
    key_buf: u8,
    key_ready: bool,
}

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
        }
    }

    pub fn keydown(&mut self, key: String) {
        if let Some(key_code) = convert_key(key.clone()) {
            self.key_buf = key_code;
            self.key_ready = true;
            info!("Got key: {}", key);
        }
    }

    pub fn keyup(&mut self, key: String) {
        if let Some(key_code) = convert_key(key) {
            if key_code == self.key_buf {
                self.key_ready = false;
                info!("Key release");
            }
        }
    }

    pub fn get_status(&self) -> u8 {
        if self.key_ready {
            return 2;
        }
        return 0;
    }

    pub fn get_keycode(&mut self) -> u8 {
        self.key_ready = false;
        return self.key_buf;
    }
}
