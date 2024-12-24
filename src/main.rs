use core::panic;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    usize,
};

fn main() {
    let data = read_file(READ_PATH_CONST).expect("File not found");
    let hashmap = create_hashmap();
    let to_binary = data
        .iter()
        .map(|f| f.split_whitespace().collect::<Vec<&str>>())
        .collect::<Vec<Vec<&str>>>();

    let parser = |x: &str| -> Result<u8, String> {
        if let Some(get) = hashmap.get(x) {
            return Ok(get.clone());
        }
        if let Ok(parsed) = x.parse() {
            return Ok(parsed);
        }
        Err(x.to_string())
    };

    let mut parse_items: Vec<Vec<Result<u8, String>>> = to_binary
        .iter()
        .map(|f| {
            f.iter()
                .map(|x| parser(x))
                .collect::<Vec<Result<u8, String>>>()
        })
        .collect();

    parse_items.iter_mut().for_each(|top| {
        let mut wait = false;
        for each in top.iter() {
            if let Err(string) = each {
                if string == WAIT.0 {
                    wait = true
                };
            }
        }
        if wait {
            *top = vec![Ok(0), Ok(0), Ok(0), Ok(0)];
        }
    });
    struct Label {
        byte_address: usize,
        vec_position: usize,
    }
    let mut keyword_counter = 0;
    let mut label_hashmap: HashMap<String, Label> = HashMap::new();
    let mut const_hashmap: HashMap<String, Label> = HashMap::new();

    for (i, each) in parse_items.iter().enumerate() {
        match (each.get(0), each.get(1), each.get(2)) {
            (Some(Err(keyword)), Some(Err(expression)), None) => {
                if keyword == "label" {
                    label_hashmap.insert(
                        expression.clone(),
                        Label {
                            byte_address: i - keyword_counter,
                            vec_position: i,
                        },
                    );
                    keyword_counter += 1;
                }
            }
            (Some(Err(keyword)), Some(Err(name)), Some(Ok(value))) => {
                if keyword == "const" {
                    let const_value: u8 = value.clone();

                    const_hashmap.insert(
                        name.clone(),
                        Label {
                            byte_address: const_value as usize,
                            vec_position: i,
                        },
                    );
                    keyword_counter += 1;
                }
            }
            _ => {}
        }
    }

    fn usize_to_u8(number: usize) -> u8 {
        number
            .try_into()
            .expect("Tried to covert label position to u8. Position was too big")
    }

    for line in parse_items.iter_mut() {
        for item in line.iter_mut() {
            if let Err(string) = item {
                match (
                    label_hashmap.get(&string.clone()),
                    const_hashmap.get(&string.clone()),
                ) {
                    (Some(label), None) => {
                        *item = Ok(usize_to_u8(label.byte_address.clone()));
                    }
                    (None, Some(const_value)) => {
                        *item = Ok(usize_to_u8(const_value.byte_address.clone()));
                    }
                    (None, None) => {}
                    _ => panic!("Multiple keywords sharing a defiition"),
                }

                // match const_hashmap.get(&string.clone()) {
                //     Some(const_value) =>
                //     None => {}
                // }*item = Ok(usize_to_u8(label.byte_address.clone())),
            }
        }
    }
    let mut remove_vec = label_hashmap
        .values()
        .into_iter()
        .map(|f| f.vec_position)
        .collect::<Vec<usize>>();

    for each in const_hashmap.values().into_iter() {
        remove_vec.push(each.vec_position);
    }

    remove_vec.sort();
    for index in remove_vec.into_iter().rev() {
        parse_items.remove(index);
    }

    fn handle_operations(f: &Vec<Result<u8, String>>) -> [u8; 4] {
        if f.len() < 4 {
            return [
                f.get(0).unwrap().clone().unwrap(),
                f.get(1).unwrap().clone().unwrap(),
                f.get(2).unwrap().clone().unwrap(),
                f.get(1).unwrap().clone().unwrap(),
            ];
            // panic!("len less than 4")
        };
        let mut found_operator: Option<(Operator, usize)> = None;
        let mut f = f.clone();
        let mut is_four = false;
        'outer_loop: loop {
            // scan for operators
            '_scan_operators: for (i, each) in f.iter().enumerate() {
                let match_op = match_operators(each.clone());
                match match_op {
                    Some(found) => {
                        found_operator = Some((found, i));
                        break '_scan_operators;
                    }
                    None => {
                        found_operator = None;
                        continue;
                    }
                };
            }
            if let Some(found_operator) = found_operator {
                // fold bytes
                let i = found_operator.1;
                match (f.get(i - 1), f.get(i), f.get(i + 1)) {
                    (Some(Ok(mut lhs)), Some(Err(_)), Some(Ok(rhs))) => {
                        // operate lhs and rhs
                        // pop operate and rhs
                        let new_lhs = &found_operator.0.operate(&lhs, *rhs);
                        lhs = new_lhs.clone();
                        f[i - 1] = Ok(lhs);
                        match f.get(i) {
                            Some(_get) => {}
                            None => println!("could not get"),
                        }
                        let _ = f.remove(i + 1);
                        let _ = f.remove(i);
                    }
                    _ => {}
                }
            } else {
                if f.len() != 4 {
                    // println!(
                    // "Wrong length: {} | {:?}\nNo more operators remaining",
                    //     f.len(),
                    //     f
                    // );
                    // panic!();
                }
            }

            if is_four {
                break 'outer_loop;
            }

            if f.len() == 4 {
                is_four = true;
            }

            // std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        if f.len() != 4 {
            println!("Something Happened");
        }
        match [f.get(0), f.get(1), f.get(2), f.get(3)] {
            [Some(Ok(a)), Some(Ok(b)), Some(Ok(c)), Some(Ok(d))] => {
                [a.clone(), b.clone(), c.clone(), d.clone()]
            }
            [Some(Ok(a)), Some(Ok(b)), Some(Ok(c)), None] => {
                [a.clone(), b.clone(), c.clone(), b.clone()]
            }
            _ => {
                println!("SOmething Happened {:?}", f);
                return [255, 255, 255, 255];
            }
        }
    }
    let opcode_byte_vec: Vec<[u8; 4]> = parse_items
        .iter()
        .map(|mut f| handle_operations(&mut f))
        .collect();

    let hex = opcode_byte_vec
        .iter()
        .map(|f| {
            f.iter()
                .map(|x| to_hex(x).iter().collect::<String>())
                .collect::<String>()
        })
        .collect::<Vec<String>>();
    // println!("hex \n{:?}", hex);
    logisim_export(WRITE_PATH_CONST, hex).unwrap();
}
enum _Token {
    U8(u8),
    STRING(String),
    Label((String, Option<u16>)),
}
fn match_operators(each: Result<u8, String>) -> Option<Operator> {
    match each {
        Ok(_) => None,
        Err(and) if and == "&" => Some(Operator::AND),
        Err(or) if or == "|" => Some(Operator::OR),
        Err(xor) if xor == "^" => Some(Operator::XOR),
        Err(not) if not == "!" => Some(Operator::NOT),
        Err(leftshift) if leftshift == "<<" => Some(Operator::LEFTSHIFT),
        Err(rightshift) if rightshift == ">>" => Some(Operator::RIGHTSHIFT),
        Err(add) if add == "+" => Some(Operator::ADD),
        Err(sub) if sub == "-" => Some(Operator::SUB),
        Err(mult) if mult == "*" => Some(Operator::MULT),
        Err(divide) if divide == "/" => Some(Operator::DIVIDE),
        Err(modulo) if modulo == "%" => Some(Operator::MODULO),
        Err(_) => None,
    }
}
#[allow(unused)]
#[derive(Clone, Copy, Debug)]
enum Operator {
    AND,
    OR,
    XOR,
    NOT,
    LEFTSHIFT,
    RIGHTSHIFT,
    ADD,
    SUB,
    MULT,
    DIVIDE,
    MODULO,
}
impl Operator {
    fn operate(&self, lhs: &u8, rhs: u8) -> u8 {
        match self {
            Operator::AND => lhs & rhs,
            Operator::OR => lhs | rhs,
            Operator::XOR => lhs ^ rhs,
            Operator::NOT => !lhs,
            Operator::LEFTSHIFT => lhs << rhs,
            Operator::RIGHTSHIFT => lhs >> rhs,
            Operator::ADD => lhs + rhs,
            Operator::SUB => lhs - rhs,
            Operator::MULT => lhs * rhs,
            Operator::DIVIDE => lhs / rhs,
            Operator::MODULO => lhs % rhs,
        }
    }
}

fn to_hex(byte: &u8) -> [char; 2] {
    let lhs = byte >> 4;
    let rhs = (byte | 0b11110000) ^ 0b11110000;
    [match_hex(&lhs), match_hex(&rhs)]
}

fn match_hex(to_hex: &u8) -> char {
    match to_hex {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'A',
        11 => 'B',
        12 => 'C',
        13 => 'D',
        14 => 'E',
        15 => 'F',
        _ => {
            println!("Error with value {to_hex} {:#10}", to_hex);
            panic!("Tried to convert u8 greater than 15 to hex");
        }
    }
}

fn create_hashmap() -> HashMap<String, u8> {
    let mut hashmap: HashMap<String, u8> = HashMap::new();
    for each in WORD_ARRAY {
        hashmap.insert(each.0.to_string(), each.1);
    }

    hashmap
}
const WORD_ARRAY: [(&str, u8); 47] = [
    MOVE,
    ADD,
    SUB,
    MULT,
    DIV,
    IM1,
    IM2,
    IMB,
    NOP,
    // WAIT,
    REGISTER_00,
    REGISTER_01,
    REGISTER_02,
    REGISTER_03,
    REGISTER_04,
    REGISTER_05,
    REGISTER_06,
    REGISTER_07,
    REGISTER_08,
    REGISTER_09,
    REGISTER_10,
    REGISTER_11,
    REGISTER_12,
    REGISTER_13,
    REGISTER_14,
    REGISTER_15,
    OP_ADD,
    OP_SUB,
    OP_MULT,
    OP_DIV,
    OP_NOT,
    OP_NAND,
    OP_AND,
    OP_OR,
    OP_XOR,
    OP_XNOR,
    OP_NOR,
    JMP_IMMEDIATELY,
    JMP_REGISTER,
    IM_JMP_REGISTER,
    JMP_ANY_REGISTER,
    JMP_ALWAYS,
    JMP_EQUAL,
    JMP_NOT_EQUAL,
    JMP_LESS,
    JMP_LESS_EQUAL,
    JMP_GREATER,
    JMP_GREATER_EQUAL,
];

const NOP: (&str, u8) = ("nop", 0b11111111);
const WAIT: (&str, u8) = ("wait", 0b00000000);
const MOVE: (&str, u8) = ("mov", 0b01000000);
const IM1: (&str, u8) = ("im1", 0b10000000);
const IM2: (&str, u8) = ("im2", 0b01000000);
const IMB: (&str, u8) = ("imb", 0b11000000);
const ADD: (&str, u8) = ("add", 0b00000000);
const SUB: (&str, u8) = ("sub", 0b00000000);
const MULT: (&str, u8) = ("mult", 0b00000000);
const DIV: (&str, u8) = ("div", 0b00000000);

const REGISTER_00: (&str, u8) = ("r0", 0b00010000);
const REGISTER_01: (&str, u8) = ("r1", 0b00010001);
const REGISTER_02: (&str, u8) = ("r2", 0b00010010);
const REGISTER_03: (&str, u8) = ("r3", 0b00010011);
const REGISTER_04: (&str, u8) = ("r4", 0b00010100);
const REGISTER_05: (&str, u8) = ("r5", 0b00010101);
const REGISTER_06: (&str, u8) = ("r6", 0b00010110);
const REGISTER_07: (&str, u8) = ("r7", 0b00010111);
const REGISTER_08: (&str, u8) = ("r8", 0b00011000);
const REGISTER_09: (&str, u8) = ("r9", 0b00011001);
const REGISTER_10: (&str, u8) = ("r10", 0b00011010);
const REGISTER_11: (&str, u8) = ("r11", 0b00011011);
const REGISTER_12: (&str, u8) = ("r12", 0b00011100);
const REGISTER_13: (&str, u8) = ("r13", 0b00011101);
const REGISTER_14: (&str, u8) = ("r14", 0b00011110);
const REGISTER_15: (&str, u8) = ("r15", 0b00011111);

const OP_ADD: (&str, u8) = ("add", 0b00000000);
const OP_SUB: (&str, u8) = ("sub", 0b00000001);
const OP_MULT: (&str, u8) = ("mult", 0b00000010);
const OP_DIV: (&str, u8) = ("div", 0b00000011);
const OP_NOT: (&str, u8) = ("not", 0b00000100);
const OP_NAND: (&str, u8) = ("nand", 0b00000101);
const OP_AND: (&str, u8) = ("and", 0b00000110);
const OP_OR: (&str, u8) = ("or", 0b00000111);
const OP_XOR: (&str, u8) = ("xor", 0b00001000);
const OP_XNOR: (&str, u8) = ("xnor", 0b00001001);
const OP_NOR: (&str, u8) = ("nor", 0b00001010);

const JMP_IMMEDIATELY: (&str, u8) = ("imj", 0b10111111);
const IM_JMP_REGISTER: (&str, u8) = ("imjr", 0b10011111);
const JMP_REGISTER: (&str, u8) = ("rjmp", 0b00011111);
const JMP_ANY_REGISTER: (&str, u8) = ("jmpany", 0b00111111);

const JMP_ALWAYS: (&str, u8) = ("jmp", 0b00010000);
const JMP_EQUAL: (&str, u8) = ("je", 0b00010001);
const JMP_NOT_EQUAL: (&str, u8) = ("jne", 0b00010010);
const JMP_LESS: (&str, u8) = ("jl", 0b00010011);
const JMP_LESS_EQUAL: (&str, u8) = ("jle", 0b00010100);
const JMP_GREATER: (&str, u8) = ("jg", 0b00010101);
const JMP_GREATER_EQUAL: (&str, u8) = ("jge", 0b00010110);

pub fn logisim_export<P: AsRef<Path> + std::fmt::Debug + Clone + Copy>(
    path: P,
    data: Vec<String>,
) -> Result<(), std::io::Error> {
    let file = File::create(path);
    let checked_file = match file {
        Ok(a) => a,
        Err(_) => File::create(path).expect("Could not create file"),
    };
    let copy = checked_file.try_clone().unwrap();
    let mut bufwriter = BufWriter::new(checked_file);
    const V2_HEADER: &str = "v2.0 raw\n";
    bufwriter.write(V2_HEADER.as_bytes()).unwrap();
    for each in data {
        bufwriter.write(each.as_bytes()).unwrap();
        bufwriter.write("\n".as_bytes()).unwrap();
    }
    println!("Wrote file to {:?}", copy);
    Ok(())
}

pub fn read_file<P: AsRef<Path> + std::fmt::Debug + Clone + Copy>(
    path: P,
) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(path);
    let checked_file = match file {
        Ok(a) => a,
        Err(_) => File::create(path).unwrap(),
    };

    let bufreader = BufReader::new(checked_file).lines();
    let string_vec: Vec<String> = bufreader.map(|f| f.expect("Buffer failed")).collect();
    // println!("Input: {:?}", string_vec);
    println!("Read {} lines from {:?}", string_vec.len(), path);
    Ok(string_vec)
}
// "C:\Users\brand\Documents\logisim\binaries"
// pub const WRITE_PATH_CONST: &str = r"C://Users/rand/Documents/logisim/binaries/simple_binary.txt";
pub const WRITE_PATH_CONST: &str = "simple_binary.txt";
// pub const WRITE_PATH_CONST: &str = "../../../../Documents/logisim/binaries/simple_binary.txt";

pub const READ_PATH_CONST: &str = "simple_assembly.txt";
