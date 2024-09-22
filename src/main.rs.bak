use core::panic;
use glob::glob;
use std::{
    env,
    fs::{read, remove_file, write, File},
    io::{self, Read, Write},
    path::Path,
};
pub mod DP2200;
use DP2200::disassembler::disassemble;

enum Sector {
    Boot(Vec<u8>),
    FileMarker(u8),
    Numeric(Vec<u8>),
    Symbolic(Vec<u8>),
    Unknown((Vec<u8>, Vec<u8>)),
}

fn get_tap_sector(data: &mut Vec<u8>) -> Option<Sector> {
    if data.len() < 4 {
        return None;
    }

    let sec_len_bytes: [u8; 4] = data[0..4].try_into().unwrap();
    data.drain(0..4);
    let sec_len = u32::from_le_bytes(sec_len_bytes) as usize;
    if data.len() < sec_len {
        return None;
    }

    let header: Vec<u8> = data.drain(0..4).collect();
    let sector: Vec<u8> = data.drain(0..sec_len - 4).collect();
    data.drain(0..4);

    if header[0..2] == [0x81, 0x7e] {
        return Some(Sector::FileMarker(header[2]));
    }

    if header[0..2] == [0xe7, 0x18] {
        println!("Symbolic");
        return Some(Sector::Symbolic(sector));
    }

    if header[0..2] == [0xc3, 0x3c] {
        return Some(Sector::Numeric(sector));
    }

    return Some(Sector::Unknown((header, sector)));
}

fn parse_tap(data: &mut Vec<u8>) -> Vec<Sector> {
    let mut sectors = Vec::new();
    let mut first_sector = true;
    while let Some(sector) = get_tap_sector(data) {
        if first_sector {
            if let Sector::Unknown((mut header, mut data)) = sector {
                let mut boot_sector = Vec::new();
                boot_sector.append(&mut header);
                boot_sector.append(&mut data);
                sectors.push(Sector::Boot(boot_sector));
            } else {
                sectors.push(sector);
            }
        } else {
            sectors.push(sector);
        }

        first_sector = false;
    }
    sectors
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(path) = args.get(1) {
        let mut outfile = path.clone();
        let mut outfile = outfile.replace(".tap", ".asm");
        remove_file(outfile.clone());
        let mut outfile = File::create(outfile).unwrap();
        let mut tap_file = read(path).unwrap();
        let tap = parse_tap(&mut tap_file);
        for sector in tap.iter() {
            match sector {
                Sector::Boot(data) => {
                    outfile.write(b"# Boot sector begin\n");
                    let disassembly = disassemble(data);
                    for (addr, inst) in disassembly {
                        outfile.write(format!("{:#04x}: {}\n", addr, inst).as_bytes());
                    }
                }
                Sector::FileMarker(file_no) => {
                    outfile.write(format!("\n\n # File no {}\n", file_no).as_bytes());
                }
                Sector::Numeric(data) => {
                    outfile.write(b"\n\n # Numeric data omitted\n");
                    let disassembly = disassemble(data);
                    for (addr, inst) in disassembly {
                        outfile.write(format!("{:#04x}: {}\n", addr, inst).as_bytes());
                    }
                }
                Sector::Symbolic(data) => {
                    outfile.write(b"# Symbolic sector begin\n");
                    let disassembly = disassemble(data);
                    for (addr, inst) in disassembly {
                        outfile.write(format!("{:#04x}: {}\n", addr, inst).as_bytes());
                    }
                }
                Sector::Unknown((header, data)) => {
                    outfile.write(
                        format!(
                            "\n\n # Unknown sector with header {:?} and len {}\n",
                            header,
                            data.len()
                        )
                        .as_bytes(),
                    );
                }
            }
        }
    }
}
