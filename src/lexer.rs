use lazy_static::lazy_static;
use regex::Regex;
use std::ops::Range;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Lexeme {
    Input,
    Output,
    LeftParentheses,
    RightParentheses,
    Plus,
    Equals,
    Loop,
    LeftCurlyBracket,
    RightCurlyBracket,
    Star,
    Comma,
    Semicolon,
    Number(u8),
    If,
    ExclamationMark,
    Minus,
    LeftArrow,
    RightArrow,
    Else,
    While,
    Break,
}

impl From<LexemeType> for Lexeme {
    fn from(arg: LexemeType) -> Self {
        match arg {
            LexemeType::Star => Lexeme::Star,
            LexemeType::Input => Lexeme::Input,
            LexemeType::Output => Lexeme::Output,
            LexemeType::LeftParentheses => Lexeme::LeftParentheses,
            LexemeType::RightParentheses => Lexeme::RightParentheses,
            LexemeType::Plus => Lexeme::Plus,
            LexemeType::Equals => Lexeme::Equals,
            LexemeType::Loop => Lexeme::Loop,
            LexemeType::LeftCurlyBracket => Lexeme::LeftCurlyBracket,
            LexemeType::RightCurlyBracket => Lexeme::RightCurlyBracket,
            LexemeType::Comma => Lexeme::Comma,
            LexemeType::Semicolon => Lexeme::Semicolon,
            LexemeType::Number => Lexeme::Number(0),
            LexemeType::If => Lexeme::If,
            LexemeType::ExclamationMark => Lexeme::ExclamationMark,
            LexemeType::Minus => Lexeme::Minus,
            LexemeType::LeftArrow => Lexeme::LeftArrow,
            LexemeType::RightArrow => Lexeme::RightArrow,
            LexemeType::Else => Lexeme::Else,
            LexemeType::While => Lexeme::While,
            LexemeType::Break => Lexeme::Break,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum LexemeType {
    Input,
    Output,
    LeftParentheses,
    RightParentheses,
    Plus,
    Equals,
    Loop,
    LeftCurlyBracket,
    RightCurlyBracket,
    Star,
    Comma,
    Semicolon,
    Number,
    If,
    ExclamationMark,
    Minus,
    LeftArrow,
    RightArrow,
    Else,
    While,
    Break,
}

impl From<Lexeme> for LexemeType {
    fn from(arg: Lexeme) -> Self {
        match arg {
            Lexeme::Star => LexemeType::Star,
            Lexeme::Input => LexemeType::Input,
            Lexeme::Output => LexemeType::Output,
            Lexeme::LeftParentheses => LexemeType::LeftParentheses,
            Lexeme::RightParentheses => LexemeType::RightParentheses,
            Lexeme::Plus => LexemeType::Plus,
            Lexeme::Equals => LexemeType::Equals,
            Lexeme::Loop => LexemeType::Loop,
            Lexeme::LeftCurlyBracket => LexemeType::LeftCurlyBracket,
            Lexeme::RightCurlyBracket => LexemeType::RightCurlyBracket,
            Lexeme::Comma => LexemeType::Comma,
            Lexeme::Semicolon => LexemeType::Semicolon,
            Lexeme::Number(_) => LexemeType::Number,
            Lexeme::If => LexemeType::If,
            Lexeme::ExclamationMark => LexemeType::ExclamationMark,
            Lexeme::Minus => LexemeType::Minus,
            Lexeme::LeftArrow => LexemeType::LeftArrow,
            Lexeme::RightArrow => LexemeType::RightArrow,
            Lexeme::Else => LexemeType::Else,
            Lexeme::While => LexemeType::While,
            Lexeme::Break => LexemeType::Break,
        }
    }
}

pub fn lex(source: &str) -> Result<Vec<Lexeme>, LexError> {
    lazy_static! {
        static ref LEXEMES: Vec<(Regex, LexemeType)> = {
            vec![
                (Regex::new("input"), LexemeType::Input),
                (Regex::new("output"), LexemeType::Output),
                (Regex::new("\\("), LexemeType::LeftParentheses),
                (Regex::new("\\)"), LexemeType::RightParentheses),
                (Regex::new("\\+"), LexemeType::Plus),
                (Regex::new("="), LexemeType::Equals),
                (Regex::new("loop"), LexemeType::Loop),
                (Regex::new("\\{"), LexemeType::LeftCurlyBracket),
                (Regex::new("\\}"), LexemeType::RightCurlyBracket),
                (Regex::new("\\*"), LexemeType::Star),
                (Regex::new("\\d+"), LexemeType::Number),
                (Regex::new(";"), LexemeType::Semicolon),
                (Regex::new("if"), LexemeType::If),
                (Regex::new("!"), LexemeType::ExclamationMark),
                (Regex::new("-"), LexemeType::Minus),
                (Regex::new("<"), LexemeType::LeftArrow),
                (Regex::new(">"), LexemeType::RightArrow),
                (Regex::new("else"), LexemeType::Else),
                (Regex::new("while"), LexemeType::While),
                (Regex::new("break"), LexemeType::Break),
            ].into_iter()
                .map(|e| (e.0.unwrap(), e.1))
                .collect()
        };
    }
    
    let mut out = Vec::new();
    let mut copied = source.to_string();
    
    while copied.len() > 0 {
        copied = copied.trim_start().to_string();
        let mut to_remove = None;
        for lexeme_pattern in &*LEXEMES {
            if let Some(t) = lexeme_pattern.0.find(&copied) {
                if t.start() == 0 {
                    to_remove = Some(t.as_str().to_string());
                    match &lexeme_pattern.1 {
                        LexemeType::Number => {
                            out.push(Lexeme::Number(t.as_str().parse::<u8>().unwrap()));
                        }
                        a => {
                            out.push(a.clone().into());
                        }
                    }
                }
            }
        }
        
        match to_remove {
            Some(t) => {
                copied.replace_range(0..t.len(), "");
            }
            None => {
                return Err(LexError::InvalidTokenError(copied));
            }
        }
    }
    
    Ok(out)
}

pub struct LexemePattern {
    characters: Vec<(LexemeMatcher, Quantity)>,
}

impl LexemePattern {
    pub fn new(characters: Vec<(LexemeMatcher, Quantity)>) -> Self {
        Self { characters }
    }
    
    pub fn matches(&self, lexemes: &[Lexeme]) -> Vec<Vec<Range<usize>>> {
        let mut out = Vec::new();
        
        let parentheses_depth =
            create_depth_table(lexemes, Lexeme::LeftParentheses, Lexeme::RightParentheses);
        let curly_brackets_depth =
            create_depth_table(lexemes, Lexeme::LeftCurlyBracket, Lexeme::RightCurlyBracket);
        
        for i in 0..lexemes.len() {
            let mut matches = true;
            let mut ptr = i;
            let mut ranges = Vec::new();
            
            for character in &self.characters {
                let depth_table = match ((character.0).1).1 {
                    DepthType::CurlyBrackets => curly_brackets_depth.clone(),
                    DepthType::Parentheses => parentheses_depth.clone(),
                    DepthType::Both => parentheses_depth.iter()
                        .zip(curly_brackets_depth.iter())
                        .map(|e| e.0 + e.1)
                        .collect(),
                };
                
                match character.1 {
                    Quantity::Finite(num) => {
                        let range = ptr..ptr + num;
                        
                        if ptr >= lexemes.len() || range.end > lexemes.len() {
                            matches = false;
                            break;
                        }
                        
                        if !lexemes[range.clone()].iter()
                            .enumerate()
                            .all(|e|
                                character.0.matches(e.1, depth_table[e.0 + ptr],
                                )) {
                            matches = false;
                            break;
                        }
                        
                        ranges.push(range);
                        ptr += num;
                    }
                    Quantity::Infinite => {
                        let start = ptr;
                        if ptr >= lexemes.len() {
                            matches = false;
                        } else if !character.0.matches(&lexemes[ptr], depth_table[ptr]) {
                            matches = false;
                        }
                        
                        loop {
                            if ptr >= lexemes.len() {
                                ranges.push(start..lexemes.len());
                                break;
                            }
                            
                            if character.0.matches(&lexemes[ptr], depth_table[ptr]) {
                                ptr += 1;
                            } else {
                                break;
                            }
                        }
                        
                        ranges.push(start..ptr);
                    }
                }
            }
            
            if matches {
                out.push(ranges);
            }
        }
        
        out
    }
}

pub struct LexemeMatcher(Box<dyn Fn(&Lexeme) -> bool + Send + Sync>, (DepthCriteria, DepthType));

impl LexemeMatcher {
    pub fn new(func: Box<dyn Fn(&Lexeme) -> bool + Send + Sync>, depth: Option<(DepthCriteria, DepthType)>) -> Self {
        LexemeMatcher(func, depth.unwrap_or((DepthCriteria::Zero, DepthType::Both)))
    }
    
    fn matches(&self, lexeme: &Lexeme, depth: i32) -> bool {
        self.0(lexeme) && match (self.1).0 {
            DepthCriteria::Zero => depth == 0,
            DepthCriteria::OneOrMore => depth >= 1,
            DepthCriteria::Any => true,
        }
    }
}

#[derive(Debug)]
pub enum Quantity {
    Finite(usize),
    Infinite,
}

#[derive(Debug)]
pub enum DepthCriteria {
    Zero,
    OneOrMore,
    Any,
}

#[derive(Debug)]
pub enum DepthType {
    Parentheses,
    CurlyBrackets,
    Both,
}

fn create_depth_table(tokens: &[Lexeme], deeper: Lexeme, shallower: Lexeme) -> Vec<i32> {
    let mut depth = Vec::with_capacity(tokens.len());
    
    let mut curr_depth = 0;
    for i in 0..tokens.len() {
        if tokens[i] == shallower {
            curr_depth -= 1;
        }
        depth.push(curr_depth);
        if tokens[i] == deeper {
            curr_depth += 1;
        }
    }
    
    depth
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LexError {
    InvalidTokenError(String),
}