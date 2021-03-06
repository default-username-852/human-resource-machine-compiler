use crate::lexer::{Lexeme, LexemePattern, LexemeMatcher, Quantity, DepthType, DepthCriteria};
use crate::{CompileError, parser2};
use crate::compiler::Command;
use std::ops::Range;
use lazy_static::lazy_static;
use matches::matches;
use crate::parser2::*;

pub fn parse_tokens(tokens: Vec<Lexeme>) -> Result<AST, CompileError> {
    AST::new(parse_tokenized_expression(tokens)?)
}

type Syntax = (LexemePattern, Box<dyn Fn(Vec<Range<usize>>, &[Lexeme]) ->
    Result<AnyExpressionType, CompileError> + Send + Sync>);

fn parse_tokenized_expression(tokens: Vec<Lexeme>) -> Result<Vec<AnyExpressionType>, CompileError> {
    lazy_static! {
        static ref EXPRESSIONS: Vec<Vec<Syntax>> = vec![
            vec![
                //Loop
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Loop)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftCurlyBracket)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::CurlyBrackets))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightCurlyBracket)), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let inner = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?.into_iter()
                        .map(|e| e.expression().unwrap())
                        .collect();
                    
                    Ok((Box::new(Loop::new(inner)) as Box<dyn Expression>).into())
                })),
                //While
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::While)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::Parentheses))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftCurlyBracket)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::CurlyBrackets))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightCurlyBracket)), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let mut exp = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    let commands = parse_tokenized_expression(tokens[t[5].clone()].to_vec())?.into_iter()
                        .map(|e| e.expression().unwrap())
                        .collect();
                    assert_eq!(exp.len(), 1);
                    
                    Ok((Box::new(While::new(
                        exp.remove(0).logical().unwrap(), commands)) as Box<dyn Expression>).into())
                })),
                //If Else
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::If)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::Parentheses))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftCurlyBracket)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::CurlyBrackets))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightCurlyBracket)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Else)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftCurlyBracket)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::CurlyBrackets))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightCurlyBracket)), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let mut exp = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    let commands1 = parse_tokenized_expression(tokens[t[5].clone()].to_vec())?.into_iter()
                        .map(|e| e.expression().unwrap())
                        .collect();
                    let commands2 = parse_tokenized_expression(tokens[t[9].clone()].to_vec())?.into_iter()
                        .map(|e| e.expression().unwrap())
                        .collect();
                    assert_eq!(exp.len(), 1);
                    
                    Ok((Box::new(IfElse::new(
                        exp.remove(0).logical().unwrap(), commands1, commands2)) as Box<dyn Expression>).into())
                })),
                //If
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::If)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::Parentheses))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftCurlyBracket)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true),
                        Some((DepthCriteria::OneOrMore, DepthType::CurlyBrackets))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightCurlyBracket)), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let mut exp = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    let commands = parse_tokenized_expression(tokens[t[5].clone()].to_vec())?.into_iter()
                        .map(|e| e.expression().unwrap())
                        .collect();
                    assert_eq!(exp.len(), 1);
                    
                    return Ok((Box::new(If::new(
                        exp.remove(0).logical().unwrap(), commands)) as Box<dyn Expression>).into());
                })),
                //Non block expressions
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::Semicolon)),
                        Some((DepthCriteria::Any, DepthType::Both))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Semicolon)), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let mut command = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    assert_eq!(command.len(), 1);
                    Ok(command.remove(0))
                })),
            ],
            vec![
                //Not equals
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::ExclamationMark)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::ExclamationMark)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Equals)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[3].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    if right[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            NotZero::new(left.remove(0).expression().unwrap())) as Box<dyn Logical>).into());
                    } else if left[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            NotZero::new(right.remove(0).expression().unwrap())) as Box<dyn Logical>).into());
                    }
                    
                    Ok((Box::new(NotZero::new(Box::new(Subtract::new(
                        left.remove(0).expression().unwrap(),
                        right.remove(0).expression().unwrap())))) as Box<dyn Logical>).into())
                })),
                //Equals
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::Equals)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Equals)), None), Quantity::Finite(2)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    if right[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            IsZero::new(left.remove(0).expression().unwrap())) as Box<dyn Logical>).into())
                    } else if left[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            IsZero::new(right.remove(0).expression().unwrap())) as Box<dyn Logical>).into())
                    }
                    Ok((Box::new(IsZero::new(Box::new(Subtract::new(
                        left.remove(0).expression().unwrap(),
                        right.remove(0).expression().unwrap())))) as Box<dyn Logical>).into())
                })),
                //Less than or equal
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::LeftArrow)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftArrow)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Equals)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[3].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    if right[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            LessOrEqualToZero::new(left.remove(0).expression().unwrap())) as Box<dyn Logical>).into())
                    }
                    Ok(AnyExpressionType::new(None, None, Some(Box::new(LessOrEqualToZero::new(Box::new(Subtract::new(
                        left.remove(0).expression().unwrap(), right.remove(0).expression().unwrap())))))))
                })),
                //Greater or equal than
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::RightArrow)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightArrow)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Equals)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[3].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    if right[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            GreaterOrEqualToZero::new(left.remove(0).expression().unwrap())) as Box<dyn Logical>).into())
                    }
                    Ok((Box::new(GreaterOrEqualToZero::new(Box::new(Subtract::new(
                        left.remove(0).expression().unwrap(),
                        right.remove(0).expression().unwrap())))) as Box<dyn Logical>).into())
                })),
                //Less than
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::LeftArrow)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftArrow)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    if right[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            LessThanZero::new(left.remove(0).expression().unwrap())) as Box<dyn Logical>).into());
                    }
                    Ok(AnyExpressionType::new(None, None, Some(Box::new(LessThanZero::new(Box::new(Subtract::new(
                        left.remove(0).expression().unwrap(), right.remove(0).expression().unwrap())))))))
                })),
                //Greater than
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::RightArrow)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightArrow)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    if right[0] == (Box::new(Number::new(0)) as Box<dyn Value>).into() {
                        return Ok((Box::new(
                            GreaterThanZero::new(left.remove(0).expression().unwrap())) as Box<dyn Logical>).into())
                    }
                    Ok(AnyExpressionType::new(None, None, Some(Box::new(GreaterThanZero::new(Box::new(Subtract::new(
                        left.remove(0).expression().unwrap(), right.remove(0).expression().unwrap())))))))
                })),
            ],
            vec![
                //Bump up
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::Plus)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Plus)), None), Quantity::Finite(2)),
                ]), Box::new(|t, tokens| {
                    let mut exp = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    assert_eq!(exp.len(), 1);
                    
                    Ok(AnyExpressionType::new(
                        Some(Box::new(Increment::new(exp.remove(0).value().unwrap()))),
                        None, None))
                })),
                //Bump down
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::Minus)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Minus)), None), Quantity::Finite(2)),
                ]), Box::new(|t, tokens| {
                    let mut exp = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    assert_eq!(exp.len(), 1);
    
                    Ok(AnyExpressionType::new(
                        Some(Box::new(Decrement::new(exp.remove(0).value().unwrap()))),
                        None, None))
                })),
            ],
            vec![
                //Assign to double deref
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Star)), None), Quantity::Finite(2)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Number(_))), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Equals)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[0..3].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[4..].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    Ok(AnyExpressionType::new(Some(Box::new(Assign::new(
                        left.remove(0).value().unwrap(), right.remove(0).expression().unwrap()))), None, None))
                })),
                //Assign to single deref
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Star)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Number(_))), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Equals)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[0..2].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[3..].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    Ok(AnyExpressionType::new(Some(Box::new(Assign::new(
                        left.remove(0).value().unwrap(), right.remove(0).expression().unwrap()))), None, None))
                })),
            ],
            vec![
                //Adding
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::Plus)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Plus)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    Ok(AnyExpressionType::new(Some(Box::new(Add::new(
                        left.remove(0).expression().unwrap(), right.remove(0).expression().unwrap()))), None, None))
                })),
                //Subtracting
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::Minus)), None), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Minus)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| true), None), Quantity::Infinite),
                ]), Box::new(|t, tokens| {
                    let mut left = parse_tokenized_expression(tokens[t[0].clone()].to_vec())?;
                    let mut right = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    assert_eq!(left.len(), 1);
                    assert_eq!(right.len(), 1);
                    
                    Ok(AnyExpressionType::new(Some(Box::new(Subtract::new(
                        left.remove(0).expression().unwrap(), right.remove(0).expression().unwrap()))), None, None))
                })),
            ],
            vec![
                //Double deref
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Star)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Star)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Number(_))), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let mut res = parse_tokenized_expression(tokens[t[1].start..t[2].end].to_vec())?;
                    assert_eq!(res.len(), 1);
                    let val = Box::new(Deref::new(res.remove(0).value().unwrap()));
                    Ok(AnyExpressionType::new(Some(val.clone()),
                                              Some(val),
                                              None))
                })),
                //Single deref
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Star)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Number(_))), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let mut res = parse_tokenized_expression(tokens[t[1].clone()].to_vec())?;
                    assert_eq!(res.len(), 1);
                    let val = Box::new(Deref::new(res.remove(0).value().unwrap()));
                    Ok(AnyExpressionType::new(Some(val.clone()),
                                              Some(val),
                                              None))
                })),
            ],
            vec![
                //Output command
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Output)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| !matches!(l, Lexeme::RightParentheses)),
                        Some((DepthCriteria::OneOrMore, DepthType::Parentheses))), Quantity::Infinite),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightParentheses)), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    let mut arg = parse_tokenized_expression(tokens[t[2].clone()].to_vec())?;
                    assert_eq!(arg.len(), 1);
                    Ok(AnyExpressionType::new(Some(Box::new(Output::new(arg.remove(0).expression().unwrap()))), None, None))
                })),
            ],
            vec![
                //Input command
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Input)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::LeftParentheses)), None), Quantity::Finite(1)),
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::RightParentheses)), None), Quantity::Finite(1)),
                ]), Box::new(|t, tokens| {
                    return Ok(AnyExpressionType::new(Some(Box::new(parser2::Input {})), None, None));
                })),
            ],
            vec![
                //Plain number
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Number(_))), None), Quantity::Finite(1))
                ]), Box::new(|t, tokens| {
                    if t[0].start == 0 {
                        return Ok(AnyExpressionType::new(None, Some(Box::new(Number::new(match tokens[0] {
                            Lexeme::Number(a) => a,
                            _ => panic!("nu blev något konstigt"),
                        }))), None));
                    }
                    Err(CompileError::InvalidCommandError(tokens.to_vec()))
                })),
                //Break
                (LexemePattern::new(vec![
                    (LexemeMatcher::new(Box::new(|l| matches!(l, Lexeme::Break)), None), Quantity::Finite(1))
                ]), Box::new(|t, tokens| {
                    Ok(AnyExpressionType::new(Some(Box::new(Break {})), None, None))
                })),
            ],
        ];
    }
    
    let mut out = Vec::new();
    let mut current_tokens = tokens;
    
    for _ in 0..50 { //1000 was chosen arbitrarily to emulate a big number, might need to be refactored at a later point
        //println!("current tokens {:?}", current_tokens);
        let mut n = 0;
        for expression_class in &*EXPRESSIONS {
            let mut earliest: Option<Vec<Range<usize>>> = None;
            let mut earliest_func = None;
            for expression in expression_class {
                let p_match = expression.0.matches(&current_tokens);
                if let Some(new) = p_match.first() {
                    if earliest.is_some() {
                        if new.first().unwrap().start < earliest.as_ref().unwrap().first().unwrap().start {
                            earliest_func = Some(&expression.1);
                            earliest = Some(new.clone());
                        }
                    } else {
                        earliest_func = Some(&expression.1);
                        earliest = Some(new.clone());
                    }
                }
                n += 1;
            }
            
            if earliest.is_some() {
                let element = earliest.unwrap();
                out.push((*earliest_func.unwrap())(element.clone(), &current_tokens)?);
                let removed = current_tokens.splice(
                    element.first().unwrap().start..element.last().unwrap().end,
                    vec![]).collect::<Vec<Lexeme>>();
                //println!("removed {:?}", removed);
                break;
            }
        }
        
        if current_tokens.len() == 0 {
            break;
        }
    }
    
    if current_tokens.len() > 0 {
        panic!("det blev över {:?}", current_tokens);
    }
    
    if out.len() > 0 {
        return Ok(out);
    }
    
    println!("hittade inget");
    Err(CompileError::InvalidCommandError(current_tokens))
}

#[derive(Debug)]
pub struct AST {
    pub root: Vec<Box<dyn Expression>>,
}

impl AST {
    fn new(expressions: Vec<AnyExpressionType>) -> Result<Self, CompileError> {
        let (ok, err): (Vec<AnyExpressionType>, Vec<AnyExpressionType>) =
            expressions.into_iter().partition(|e| e.is_expression());
        
        if err.len() > 0 {
            return Err(CompileError::Error); //TODO: make me good
        }
        Ok(Self { root: ok.into_iter()
            .map(|e| e.expression().unwrap())
            .collect()})
    }
    
    fn to_commands(&self, add_addr: u8) -> Result<Vec<Command>, CompileError> { //TODO: use me
        let mut label_counter = 0;
        
        let (ok, err): (Vec<_>, Vec<_>) = self.root.iter()
            .map(|e| e.to_command(&mut label_counter, add_addr, None))
            .partition(|e| e.is_ok());
    
        for e in err {
            return e;
        }
        
        Ok(ok.into_iter()
            .map(|e| e.unwrap())
            .flatten()
            .collect())
    }
}

/*#[derive(Debug)]
pub enum Expression {
    Number(u8),
    Deref(Box<Expression>),
    Output(Box<Expression>),
    Input,
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Loop(Vec<Expression>),
    Assign(Box<Expression>, Box<Expression>),
    If(Box<Expression>, Vec<Expression>),
    IfElse(Box<Expression>, Vec<Expression>, Vec<Expression>),
    IsZero(Box<Expression>),
    NotZero(Box<Expression>),
    GreaterThanZero(Box<Expression>),
    LessThanZero(Box<Expression>),
    GreaterOrEqualToZero(Box<Expression>),
    LessOrEqualToZero(Box<Expression>),
    While(Box<Expression>, Vec<Expression>),
    Increment(Box<Expression>),
    Decrement(Box<Expression>),
    Break,
}

impl Expression {
    pub fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> String {
        match self {
            Expression::Input => return String::from("\tINBOX\t\n"),
            Expression::Number(num) => {
                if num == &0 {
                    return String::from("");
                }
                println!("de e numberwang att lägga ut en siffra direkt")
            },
            Expression::Deref(n) => {
                if let Expression::Number(num) = &**n {
                    return format!("\tCOPYFROM\t{}\n", num);
                } else if let Expression::Deref(n2) = &**n {
                    if let Expression::Number(num) = &**n2 {
                        return format!("\tCOPYFROM\t[{}]\n", num);
                    } else {
                        panic!("oops 2 electric boogaloo");
                    }
                } else {
                    panic!("oops");
                }
            }
            Expression::Output(c) => return format!("{}\tOUTBOX\t\n", c.to_command(label_counter, add_addr)),
            Expression::Add(l, r) => {
                let mut res = l.to_command(label_counter, add_addr);
                res.extend(format!("\tCOPYTO\t{1}\n{}\tADD\t{1}\n", r.to_command(label_counter, add_addr), add_addr).chars());
                return res;
            }
            Expression::Loop(inner) => {
                let label_0 = create_label(label_counter);
                let res = format!("{0}:\n{1}\tJUMP\t{0}\n", label_0, inner.iter()
                    .map(|e| e.to_command(label_counter, add_addr))
                    .collect::<String>());
                return res;
            }
            Expression::Assign(l, r) => {
                let mut res = String::new();
                res.extend(r.to_command(label_counter, add_addr).chars());
                if let Expression::Deref(n) = &**l {
                    if let Expression::Number(num) = &**n {
                        res.extend(format!("\tCOPYTO\t{}\n", num).chars());
                    } else if let Expression::Deref(n2) = &**n {
                        if let Expression::Number(num) = &**n2 {
                             res.extend(format!("\tCOPYTO\t[{}]\n", num).chars());
                        } else {
                            panic!("oops 2 electric boogaloo");
                        }
                    } else {
                        panic!("oops");
                    }
                }
                return res;
            }
            Expression::If(exp, content) => {
                let mut res = String::new();
                let end_label = create_label(label_counter);
    
                if let Expression::IsZero(e) = &**exp {
                    let label_1 = create_label(label_counter);
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPZ\t{0}\n\tJUMP\t{1}\n{0}:\n", label_1, end_label).chars());
                } else if let Expression::NotZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPZ\t{}\n", end_label).chars());
                } else if let Expression::LessThanZero(e) = &**exp {
                    let label_1 = create_label(label_counter);
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n\tJUMP\t{1}\n{0}:\n", label_1, end_label).chars());
                } else if let Expression::GreaterThanZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPZ\t{0}\n\tJUMPN\t{0}\n", end_label).chars());
                } else if let Expression::GreaterOrEqualToZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n", end_label).chars());
                } else if let Expression::LessOrEqualToZero(e) = &**exp {
                    let short_jump = create_label(label_counter);
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n\tJUMPZ\t{0}\n\tJUMP\t{1}\n{0}:\n", short_jump, end_label).chars());
                } else {
                    res.extend("aaaa\n".chars());
                }
    
                res.extend(content.iter()
                    .map(|e| e.to_command(label_counter, add_addr))
                    .collect::<String>().chars());
                res.extend(format!("{}:\n", end_label).chars());
                
                return res;
            }
            Expression::IfElse(exp, c1, c2) => {
                let mut res = String::new();
                let block1_commands = c1.iter()
                    .map(|e| e.to_command(label_counter, add_addr))
                    .collect::<String>();
                let block2_commands = c2.iter()
                    .map(|e| e.to_command(label_counter, add_addr))
                    .collect::<String>();
    
                let label_0 = create_label(label_counter);
                let label_1 = create_label(label_counter);
                
                if let Expression::IsZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPZ\t{0}\n{2}\tJUMP\t{1}\n{0}:\n{3}{1}:\n",
                                       label_0, label_1, block2_commands, block1_commands).chars());
                } else if let Expression::NotZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPZ\t{0}\n{2}\tJUMP\t{1}\n{0}:\n{3}{1}:\n",
                                       label_0, label_1, block1_commands, block2_commands).chars());
                } else if let Expression::LessThanZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n{2}\tJUMP\t{1}\n{0}:\n{3}{1}:\n",
                                       label_0, label_1, block2_commands, block1_commands).chars());
                } else if let Expression::GreaterThanZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n\tJUMPZ\t{0}\n{2}\tJUMP\t{1}\n{0}:\n{3}{1}:\n",
                                       label_0, label_1, block1_commands, block2_commands).chars());
                } else if let Expression::GreaterOrEqualToZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n{2}\tJUMP\t{1}\n{0}:\n{3}{1}:\n",
                                       label_0, label_1, block1_commands, block2_commands).chars());
                } else if let Expression::LessOrEqualToZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n\tJUMPZ\t{0}\n{3}\tJUMP\t{1}\n{0}:\n{2}{1}:\n",
                                       label_0, label_1, block1_commands, block2_commands).chars());
                } else {
                    res.extend("aaaa\n".chars());
                }
    
                return res;
            }
            Expression::Subtract(l, r) => {
                let mut res = r.to_command(label_counter, add_addr);
                res.extend(format!("\tCOPYTO\t{1}\n{}\tSUB\t{1}\n", l.to_command(label_counter, add_addr), add_addr).chars());
                return res;
            }
            Expression::IsZero(_) => { println!("this shouldn't happen"); }
            Expression::NotZero(_) => { println!("this shouldnt happen"); }
            Expression::GreaterThanZero(_) => { println!("this shouldnt happen"); }
            Expression::LessThanZero(_) => { println!("this shouldnt happen"); }
            Expression::GreaterOrEqualToZero(_) => { println!("this shouldn't happen"); }
            Expression::LessOrEqualToZero(_) => { println!("this shouldn't happen"); }
            Expression::While(exp, commands) => {
                let top_label = create_label(label_counter);
                let bottom_label = create_label(label_counter);
                let mut res = String::new();
    
                res.extend(format!("{}:\n", top_label).chars());
                if let Expression::IsZero(e) = &**exp {
                    let small_jump = create_label(label_counter);
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPZ\t{0}\n\tJUMP\t{1}\n{0}:\n", small_jump, bottom_label).chars());
                } else if let Expression::NotZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPZ\t{0}\n", bottom_label).chars());
                } else if let Expression::LessThanZero(e) = &**exp {
                    let small_jump = create_label(label_counter);
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n\tJUMP\t{1}\n{0}:\n", small_jump, bottom_label).chars());
                } else if let Expression::GreaterThanZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n\tJUMPZ\t{0}\n", bottom_label).chars());
                } else if let Expression::GreaterOrEqualToZero(e) = &**exp {
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n", bottom_label).chars());
                } else if let Expression::LessOrEqualToZero(e) = &**exp {
                    let short_jump = create_label(label_counter);
                    res.extend(e.to_command(label_counter, add_addr).chars());
                    res.extend(format!("\tJUMPN\t{0}\n\tJUMPZ\t{0}\n\tJUMP\t{1}\n{0}:\n", short_jump, bottom_label).chars());
                } else {
                    res.extend("aaaa\n".chars());
                }
                
                res.extend(commands.iter()
                    .map(|e| e.to_command(label_counter, add_addr))
                    .collect::<String>().chars());
                res.extend(format!("\tJUMP\t{}\n{}:\n", top_label, bottom_label).chars());
                
                return res;
            }
            Expression::Increment(exp) => {
                let mut res = String::new();
                if let Expression::Deref(num) = &**exp {
                    if let Expression::Number(n) = &**num {
                        res.extend(format!("\tBUMPUP\t{}\n", n).chars());
                    } else {
                        panic!("oops");
                    }
                }
                return res;
            }
            Expression::Decrement(exp) => {
                let mut res = String::new();
                if let Expression::Deref(num) = &**exp {
                    if let Expression::Number(n) = &**num {
                        res.extend(format!("\tBUMPDN\t{}\n", n).chars());
                    } else {
                        panic!("oops");
                    }
                }
                return res;
            }
            Expression::Break => {}
        }
        
        String::from("test\n")
    }
}*/

/*
fn create_label(num: &mut u8) -> String {
    let res = number_to_chars(num);
    *num += 1;
    res
}

fn number_to_chars(number: &u8) -> String {
    let small = number % 16;
    let big = number / 16;
    [(big + 97) as char, (small + 97) as char].iter().collect()
}*/
