mod preprocessor;
mod lexer;
mod parser;
mod parser2;
mod compiler;

use std::fs::File;
use std::io::{Read, Write};
use crate::preprocessor::{parse_macros, trim, find_add_square};
use crate::lexer::{lex, Lexeme};
use crate::parser::parse_tokens;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CompileError {
    Error,
    InvalidCommandError(Vec<Lexeme>),
    InvalidToAssignTo(Vec<Lexeme>),
    NumberInsertionError,
}

fn main() {
    let mut read = String::new();
    let mut in_file = File::open("in.txt").unwrap(); //TODO: make this safe

    in_file.read_to_string(&mut read).unwrap();
    
    read = parse_macros(read);
    let (mut read, add_square) = find_add_square(read);
    println!("parsed macros");
    read = trim(read);
    println!("trimmed");
    
    let lexed = lex(&read).unwrap();
    println!("lexed {:?}", lexed);
    let parsed = parse_tokens(lexed).unwrap();
    println!("parsed");

    println!("{:?}", parsed);
    
    let mut output = String::from("-- HUMAN RESOURCE MACHINE PROGRAM --\n\n");

    let mut label_counter = 0;
    for command in &parsed.root {
        output.extend(command.to_command(&mut label_counter, add_square, None).unwrap()
            .into_iter()
            .map(|e| e.to_string()));
    }

    let mut out_file = File::create("out.txt").unwrap();

    out_file.write(output.as_bytes()).unwrap();
}