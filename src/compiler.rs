use std::fmt::{Display, Formatter, Error};

#[derive(Debug)]
pub enum Command<'a> {
    Inbox,
    Outbox,
    CopyTo(Reference),
    CopyFrom(Reference),
    Add(Reference),
    Subtract(Reference),
    Increment(Reference),
    Decrement(Reference),
    Jump(&'a Label),
    JumpIfZero(&'a Label),
    JumpIfNegative(&'a Label),
    Label(Label),
}

impl<'a> Display for Command<'a> {
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
            Command::Jump(label) => format!("JUMP\t{}", label.value()),
            Command::JumpIfZero(label) => format!("JUMPZ\t{}", label.value()),
            Command::JumpIfNegative(label) => format!("JUMPN\t{}", label.value()),
            Command::Label(label) => format!("{}", label),
        })
    }
}

#[derive(Debug)]
pub enum Reference {
    Literal(u8),
    Pointer(u8),
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Reference::Literal(num) => write!(f, "{}", num),
            Reference::Pointer(num) => write!(f, "[{}]", num),
        }
    }
}

#[derive(Debug)]
pub struct Label {
    count: u8,
}

impl Label {
    fn new(count: u8) -> Self {
        Self { count }
    }
    
    fn value(&self) -> String {
        let small = self.count % 16;
        let big = self.count / 16;
        [(big + 97) as char, (small + 97) as char].iter().collect()
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}:\n", self.value())
    }
}

