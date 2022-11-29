#![allow(dead_code)]
use parse_int::parse;
use std::vec;

fn get_instruction_byte_size(inst: &str) -> u16 {
    match inst {
        "LoadImm" => 2,
        "Load" => 1,
        "AddImm" => 2,
        "Add" => 1,
        "AddImmCarry" => 2,
        "AddCarry" => 1,
        "SubImm" => 2,
        "Sub" => 1,
        "SubImmBorrow" => 2,
        "SubBorrow" => 1,
        "AndImm" => 2,
        "And" => 1,
        "OrImm" => 2,
        "Or" => 1,
        "XorImm" => 2,
        "Xor" => 1,
        "CompImm" => 2,
        "Comp" => 1,
        "Jump" => 3,
        "JumpIf" => 3,
        "JumpIfNot" => 3,
        "Call" => 3,
        "CallIf" => 3,
        "CallIfNot" => 3,
        "Return" => 1,
        "ReturnIf" => 1,
        "ReturnIfNot" => 1,
        "ShiftRight" => 1,
        "ShiftLeft" => 1,
        "Nop" => 1,
        "Halt" => 1,
        "Input" => 1,
        "Pop" => 1,
        "Push" => 1,
        "EnableIntr" => 1,
        "DisableInts" => 1,
        "SelectAlpha" => 1,
        "SelectBeta" => 1,
        "Ex" => 1,
        _ => panic!("Unknown instruction {}", inst),
    }
}

// Take a string, and removes everything after the first #
// Example:
// Label1: Add 2, 3 # This is a comment -> Label1: Add 2, 3
fn remove_comment(line: &mut String) {
    match line.find('#') {
        Some(index) => line.replace_range(index.., ""),
        None => (),
    }
}

// Removes any space from the beginning and ends of a string.
// It both mutates inplace and returns the result
fn strip(line: &mut String) -> &mut String {
    while line.starts_with(' ') {
        line.remove(0);
    }

    while line.ends_with(' ') {
        line.remove(line.len() - 1);
    }

    line
}

// Takes a line of assemble and returns an optional label or address.
// Example:
// Label1: Add 2, 3 -> Some("Label1"), None
// 0050: Add 2, 3   -> None, Some(50)
fn get_label_or_address(line: &mut String) -> (Option<String>, Option<u16>) {
    if let Some(index) = line.find(':') {
        let address_label = line[..index].to_owned();
        line.replace_range(0..=index, "");
        match parse::<u16>(&address_label) {
            Ok(i) => return (None, Some(i)),
            Err(_) => return (Some(address_label), None),
        };
    }

    (None, None)
}

// Take a line of assemble, without label or leading/trailing whitespace and returns the instruction name
// Example:
// Add 2, 3 -> Add
fn get_instruction(line: &str) -> &str {
    if line.contains(':') {
        panic!("Label was not removed before calling get_instruction")
    }

    if line.starts_with(' ') {
        panic!("Assembly contains leading whitespace when calling get_instructions");
    }

    if line.contains(' ') {
        return &line[..line.find(' ').unwrap()];
    }

    line
}

// Takes a line of assembly and returns a list of operands
// The label and any leading/trailing whitespace should already be removed
// Example:
// Add 2, 3 -> ["2", "3"]

#[derive(Debug)]
struct OpParser {
    ops: Vec<String>,
    counter: usize,
}

const REGISTER_NAMES: [(&str, &str); 12] = [
    ("A", "0"),
    ("B", "1"),
    ("C", "2"),
    ("D", "3"),
    ("E", "4"),
    ("H", "5"),
    ("L", "6"),
    ("M", "7"),
    ("Cf", "0"),
    ("Zf", "1"),
    ("Sf", "2"),
    ("Pf", "3"),
];

impl OpParser {
    fn new(line: &str, label_list: &[(String, u16)]) -> OpParser {
        let inst = get_instruction(line);
        let mut operands = line.to_owned();
        operands.replace_range(..inst.len(), "");
        let op_list = operands.split(',').map(|x| x.to_owned());
        let mut res = Vec::new();

        for mut op in op_list {
            strip(&mut op);
            for (label, address) in label_list.iter() {
                if label.eq(&op) {
                    op.replace_range(.., &address.to_string());
                }
            }

            for rn in REGISTER_NAMES.iter() {
                if rn.0.eq(&op) {
                    op.replace_range(.., rn.1);
                }
            }
            res.push(op);
        }

        OpParser {
            ops: res,
            counter: 0,
        }
    }

    fn op(&mut self) -> u8 {
        self.counter += 1;
        parse::<u8>(&self.ops[self.counter - 1]).unwrap()
    }

    fn lsp(&self) -> u8 {
        (parse::<u16>(&self.ops[self.counter]).unwrap() & 0xff) as u8
    }

    fn msp(&mut self) -> u8 {
        self.counter += 1;
        (parse::<u16>(&self.ops[self.counter - 1]).unwrap() >> 8) as u8
    }
}

#[rustfmt::skip]
fn parse_instruction(line: &str, label_list: &[(String, u16)]) -> Vec<u8> {
    let inst = get_instruction(line);
    let mut op = OpParser::new(line, label_list);

    // Used to set the [t]ype, [d]estination and [s]ource of the opcode
    let tds = |t: u8, d: u8, s: u8| (t & 3) << 6 | (d & 7) << 3 | s & 7;
    match inst {           
        "Halt"         => vec![0],
        "Load"         => vec![tds(3, op.op(), op.op())],
        "Add"          => vec![tds(2, 0, op.op())],
        "AddCarry"     => vec![tds(2, 1, op.op())],
        "Sub"          => vec![tds(2, 2, op.op())],
        "SubBorrow"    => vec![tds(2, 3, op.op())],
        "And"          => vec![tds(2, 4, op.op())],
        "Or"           => vec![tds(2, 6, op.op())],
        "Xor"          => vec![tds(2, 5, op.op())],
        "Comp"         => vec![tds(2, 7, op.op())],
        "ReturnIf"     => vec![tds(0, op.op() + 4, 3)],
        "ReturnIfNot"  => vec![tds(0, op.op(), 3)],
        "Return"       => vec![tds(0, 0, 7)],
        "ShiftRight"   => vec![tds(0, 1, 2)],
        "ShiftLeft"    => vec![tds(0, 0, 2)],
        "Nop"          => vec![tds(3, 0, 0)],
        "Input"        => vec![tds(1, 0, 1)],
        "Pop"          => vec![tds(0, 6, 0)],
        "Push"         => vec![tds(0, 7, 0)],
        "EnableIntr"   => vec![tds(0, 5, 0)],
        "DisableInts"  => vec![tds(0, 4, 0)],
        "SelectAlpha"  => vec![tds(0, 3, 0)],

        // Immediate instructions
        "LoadImm"      => vec![tds(0, op.op(), 6)    , op.op()],
        "AddImm"       => vec![tds(0, 0, 4)          , op.op()],
        "AddImmCarry"  => vec![tds(0, 1, 4)          , op.op()],
        "SubImm"       => vec![tds(0, 2, 4)          , op.op()],
        "SubImmBorrow" => vec![tds(0, 3, 4)          , op.op()],
        "AndImm"       => vec![tds(0, 4, 4)          , op.op()],
        "OrImm"        => vec![tds(0, 6, 4)          , op.op()],
        "XorImm"       => vec![tds(0, 5, 4)          , op.op()],
        "CompImm"      => vec![tds(0, 7, 4)          , op.op()],

        // Instructions using 16 bit address as operand
        "Jump"         => vec![tds(1, 0, 4)          , op.lsp(), op.msp()],
        "JumpIf"       => vec![tds(1, op.op() + 4, 0), op.lsp(), op.msp()],
        "JumpIfNot"    => vec![tds(1, op.op(), 0)    , op.lsp(), op.msp()],
        "Call"         => vec![tds(1, 0, 6)          , op.lsp(), op.msp()],
        "CallIf"       => vec![tds(1, op.op() + 4, 2), op.lsp(), op.msp()],
        "CallIfNot"    => vec![tds(1, op.op(), 2)    , op.lsp(), op.msp()],

        // Ex commands are defined using octal codes form reference manual
        "Adr"          => vec![0o121],
        "Status"       => vec![0o123],
        "Data"         => vec![0o125],
        "Write"        => vec![0o127],
        "Com1"         => vec![0o131],
        "Com2"         => vec![0o133],
        "Com3"         => vec![0o135],
        "Com4"         => vec![0o137],
        "Beep"         => vec![0o151],
        "Click"        => vec![0o153],
        "Deck1"        => vec![0o155],
        "Deck2"        => vec![0o157],
        "Rbk"          => vec![0o161],
        "Wbk"          => vec![0o163],
        "Bsp"          => vec![0o167],
        "Sf"           => vec![0o171],
        "Sb"           => vec![0o173],
        "Rewind"       => vec![0o175],
        "Tstop"        => vec![0o177],
        i => panic!("Unknown instruction {}", i),
    }
}

// Take an iterator of lines of assembly and generates a list of (label, address) pairs
// that can be used in a second pass to generate the final bytecode
fn parse_label_list<'a>(lines: impl Iterator<Item = &'a str>) -> Vec<(String, u16)> {
    let mut current_address: u16 = 0;
    let mut label_list = Vec::new();
    for line in lines {
        let mut line = String::from(line);
        remove_comment(&mut line);
        strip(&mut line);
        if line.is_empty() {
            continue;
        }
        let (label, address) = get_label_or_address(&mut line);
        if let Some(label) = label {
            label_list.push((label.to_owned(), current_address));
        }

        if let Some(address) = address {
            if address <= current_address {
                panic!("Invalid address {} at {}", address, current_address);
            }
            current_address = address;
        }

        strip(&mut line);
        current_address += get_instruction_byte_size(get_instruction(&line));
    }

    label_list
}

// Insert element at index, if index is  larger than len of vec, inserts zeros
fn insert_and_extend(l: &mut Vec<u8>, element: u8, index: usize) {
    if index < l.len() {
        panic!(
            "Inserting into list regularly using this function is not a good idea {}, {}",
            index,
            l.len()
        );
    }

    while l.len() < index {
        l.push(0);
    }

    l.insert(index, element);
}

pub fn assemble(lines: Vec<&str>) -> Vec<u8> {
    let label_list = parse_label_list(lines.clone().into_iter());
    let mut res = Vec::new();
    let mut current_address: u16 = 0;
    for line in lines {
        let mut line = String::from(line);
        remove_comment(&mut line);
        strip(&mut line);
        if line.is_empty() {
            continue;
        }
        let (_, address) = get_label_or_address(&mut line);
        if let Some(address) = address {
            current_address = address;
        }

        strip(&mut line);
        let inst = parse_instruction(&line, &label_list);
        for b in inst.into_iter() {
            insert_and_extend(&mut res, b, current_address as usize);
            current_address += 1;
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_comment() {
        let mut s = String::from("Labe1: Add # This is a comment");
        remove_comment(&mut s);
        assert_eq!(s.as_str(), "Labe1: Add ");
    }

    #[test]
    fn test_strip() {
        let mut s = String::from("  Test ");
        strip(&mut s);
        assert_eq!(s.as_str(), "Test");
    }

    #[test]
    fn test_get_label() {
        let mut s = String::from("Label1: Add # This is a comment");
        let (label, address) = get_label_or_address(&mut s);
        remove_comment(&mut s);
        strip(&mut s);
        assert_eq!(s.as_str(), "Add");
        assert_eq!(label, Some(String::from("Label1")));
        assert_eq!(address, None);
    }

    #[test]
    fn test_get_address() {
        let mut s = String::from("0050: Add # This is a comment");
        let (label, address) = get_label_or_address(&mut s);
        remove_comment(&mut s);
        strip(&mut s);
        assert_eq!(s.as_str(), "Add");
        assert_eq!(label, None);
        assert_eq!(address, Some(50));
    }

    #[test]
    fn test_get_inst() {
        let mut s = String::from("0050: Add 2, 3# This is a comment");
        let (label, address) = get_label_or_address(&mut s);
        remove_comment(&mut s);
        strip(&mut s);
        assert_eq!(s.as_str(), "Add 2, 3");
        assert_eq!(get_instruction(&s), "Add");
        assert_eq!(label, None);
        assert_eq!(address, Some(50));
    }

    #[test]
    fn test_pass_one() {
        let program = vec![
            "0050: Add 2",
            "label1: Sub 2",
            "LoadImm",
            "# This line is only a comment",
            "label2: Halt",
        ];
        let label_list = parse_label_list(program.into_iter());
        assert_eq!(label_list[0].0, String::from("label1"));
        assert_eq!(label_list[0].1, 51);

        assert_eq!(label_list[1].0, String::from("label2"));
        assert_eq!(label_list[1].1, 54);
    }

    #[test]
    fn test_parse_operands() {
        let l = String::from("Add 2, 3");
        let label_list = Vec::new();
        let mut op = OpParser::new(&l, &label_list);
        assert_eq!(op.op(), 2);
        assert_eq!(op.op(), 3);
    }

    #[test]
    fn test_parse_operands_with_label() {
        let l = String::from("Add 2, label1");
        let label_list = vec![(String::from("label1"), 3)];
        let mut op = OpParser::new(&l, &label_list);
        assert_eq!(op.op(), 2);
        assert_eq!(op.op(), 3);
    }

    #[test]
    fn test_parse_load() {
        let l = String::from("Load 2, 3");
        let label_list = Vec::new();
        let res = parse_instruction(&l, &label_list);
        assert_eq!(res[0], 0xd3);
    }

    #[test]
    fn test_parse_load_imm() {
        let l = String::from("LoadImm 2, 3");
        let label_list = Vec::new();
        let res = parse_instruction(&l, &label_list);
        assert_eq!(res[0], 0x16);
        assert_eq!(res[1], 3);
    }

    #[test]
    fn test_parse_jump() {
        let l = String::from("Jump 0x0f0f");
        let label_list = Vec::new();
        let res = parse_instruction(&l, &label_list);
        assert_eq!(res[0], 0x44);
        assert_eq!(res[1], 0xf);
        assert_eq!(res[2], 0xf);
    }

    #[test]
    fn test_parse_program() {
        let program = vec!["Add C", "label1: Sub 2", "10: Jump label1"];
        let res = assemble(program);
        assert_eq!(res, vec![0x82, 0x92, 0, 0, 0, 0, 0, 0, 0, 0, 0x44, 1, 0]);
    }
}
