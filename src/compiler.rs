use std::fmt::{Display, Formatter, Error};
use std::fmt;

#[derive(Debug)]
pub enum Command {
    Inbox,
    Outbox,
    CopyTo(Reference),
    CopyFrom(Reference),
    Add(Reference),
    Subtract(Reference),
    Increment(Reference),
    Decrement(Reference),
    Jump(LabelRef),
    JumpIfZero(LabelRef),
    JumpIfNegative(LabelRef),
    Label(Label),
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "\t{}\n", match self {
            Command::Inbox => "INBOX\t".to_string(),
            Command::Outbox => "OUTBOX\t".to_string(),
            Command::CopyTo(ptr) => format!("COPYTO\t{}", ptr),
            Command::CopyFrom(ptr) => format!("COPYFROM\t{}", ptr),
            Command::Add(ptr) => format!("ADD\t{}", ptr),
            Command::Subtract(ptr) => format!("SUB\t{}", ptr),
            Command::Increment(ptr) => format!("BUMPUP\t{}", ptr),
            Command::Decrement(ptr) => format!("BUMPDN\t{}", ptr),
            Command::Jump(label) => format!("JUMP\t{}", label),
            Command::JumpIfZero(label) => format!("JUMPZ\t{}", label),
            Command::JumpIfNegative(label) => format!("JUMPN\t{}", label),
            Command::Label(label) => format!("{}", label),
        })
    }
}

#[derive(Debug)]
pub enum Reference {
    Number(u8),
    Pointer(u8),
    PointerPointer(u8),
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Reference::Pointer(num) => write!(f, "{}", num),
            Reference::Number(num) => Err(fmt::Error),
            Reference::PointerPointer(num) => write!(f, "[{}]", num),
        }
    }
}

#[derive(Debug)]
pub struct Label {
    count: u8,
}

impl Label {
    pub fn new(count: &mut u8) -> Self {
        let out = Self { count: *count };
        *count += 1;
        out
    }
    
    pub fn reference(&self) -> LabelRef {
        LabelRef::new(self)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}:", number_to_chars(&self.count))
    }
}

#[derive(Debug, Clone)]
pub struct LabelRef {
    count: u8
}

impl LabelRef {
    pub fn new(label: &Label) -> Self {
        Self { count: label.count }
    }
}

impl Display for LabelRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", number_to_chars(&self.count))
    }
}

fn number_to_chars(number: &u8) -> String {
    let small = number % 16;
    let big = number / 16;
    [(big + 97) as char, (small + 97) as char].iter().collect()
}