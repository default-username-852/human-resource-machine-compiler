use std::fmt::Debug;
use crate::compiler::{Command, Reference, Label, LabelRef};
use crate::CompileError;
use downcast_rs::Downcast;
use downcast_rs::impl_downcast;
use crate::compiler::Command::CopyTo;

pub trait Expression: Debug + Downcast {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError>;
    fn eq(&self, other: &dyn Expression) -> bool;
}
impl_downcast!(Expression);

impl PartialEq for dyn Expression {
    fn eq(&self, other: &dyn Expression) -> bool {
        Expression::eq(self, other)
    }
}

impl Eq for dyn Expression {}

pub trait Value: Debug + Downcast {
    fn value(&self) -> Result<Reference, CompileError>;
    fn eq(&self, other: &dyn Value) -> bool;
}
impl_downcast!(Value);

impl PartialEq for dyn Value {
    fn eq(&self, other: &Self) -> bool {
        Value::eq(self, other)
    }
}

impl Eq for dyn Value {}

pub trait Logical: Debug + Downcast {
    fn to_commands(&self, end_label: LabelRef, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError>;
    fn eq(&self, other: &dyn Logical) -> bool;
}
impl_downcast!(Logical);

impl PartialEq for dyn Logical {
    fn eq(&self, other: &Self) -> bool {
        Logical::eq(self, other)
    }
}

impl Eq for dyn Logical {}

#[derive(PartialEq, Eq, Debug)]
pub struct AnyExpressionType {
    expression: Option<Box<dyn Expression>>,
    value: Option<Box<dyn Value>>,
    logical: Option<Box<dyn Logical>>,
}

impl AnyExpressionType {
    pub fn new(expression: Option<Box<dyn Expression>>, value: Option<Box<dyn Value>>, logical: Option<Box<dyn Logical>>) -> Self {
        /*assert_eq!(expression.as_ref().clone().map(|_| 1).unwrap_or(0) +
            value.as_ref().clone().map(|_| 1).unwrap_or(0) +
            logical.as_ref().clone().map(|_| 1).unwrap_or(0), 1);*/
        
        
        Self { expression, value, logical }
    }
    
    pub fn expression_ref(&self) -> &Option<Box<dyn Expression>> {
        &self.expression
    }
    
    pub fn value_ref(&self) -> &Option<Box<dyn Value>> {
        &self.value
    }
    
    pub fn logical_ref(&self) -> &Option<Box<dyn Logical>> {
        &self.logical
    }
    
    pub fn expression(self) -> Option<Box<dyn Expression>> {
        self.expression
    }
    
    pub fn value(self) -> Option<Box<dyn Value>> {
        self.value
    }
    
    pub fn logical(self) -> Option<Box<dyn Logical>> {
        self.logical
    }
    
    pub fn is_expression(&self) -> bool {
        self.expression.is_some()
    }
    
    pub fn is_value(&self) -> bool {
        self.value.is_some()
    }
    
    pub fn is_logical(&self) -> bool {
        self.logical.is_some()
    }
}

impl From<Box<dyn Expression>> for AnyExpressionType {
    fn from(val: Box<dyn Expression>) -> Self {
        Self {
            expression: Some(val),
            value: None,
            logical: None
        }
    }
}

impl From<Box<dyn Value>> for AnyExpressionType {
    fn from(val: Box<dyn Value>) -> Self {
        Self {
            expression: None,
            value: Some(val),
            logical: None
        }
    }
}

impl From<Box<dyn Logical>> for AnyExpressionType {
    fn from(val: Box<dyn Logical>) -> Self {
        Self {
            expression: None,
            value: None,
            logical: Some(val)
        }
    }
}

macro_rules! impl_eq {
    ($scope:ident) => {
        fn eq(&self, other: &dyn $scope) -> bool {
            match other.downcast_ref::<Self>() {
                Some(t) => t == self,
                None => false,
            }
        }
    };
}

macro_rules! impl_partialeq {
    ($object:ident, $($params:ident),+) => {
        impl PartialEq for $object {
            fn eq(&self, other: &Self) -> bool {
                $(&self.$params == &other.$params)&&+
            }
        }
        
        impl Eq for $object {}
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct Number {
    number: u8,
}

impl Number {
    pub fn new(number: u8) -> Self {
        Self { number }
    }
}

impl Value for Number {
    fn value(&self) -> Result<Reference, CompileError> {
        Ok(Reference::Number(self.number))
    }
    
    impl_eq!(Value);
}

#[derive(Debug)]
pub struct Output {
    argument: Box<dyn Expression>,
}

impl_partialeq!(Output, argument);

impl Output {
    pub fn new(argument: Box<dyn Expression>) -> Self {
        Self { argument }
    }
}

impl Expression for Output {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(self.argument.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::Outbox])
            .collect())
    }
    
    impl_eq!(Expression);
}

#[derive(Debug, PartialEq, Eq)]
pub struct Input {}

impl Expression for Input {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(vec![Command::Inbox])
    }
    
    impl_eq!(Expression);
}

#[derive(Debug)]
pub struct Deref {
    to_deref: Box<dyn Value>,
}

impl Deref {
    pub fn new(to_deref: Box<dyn Value>) -> Self {
        Self { to_deref }
    }
}

impl Expression for Deref {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        return Ok(vec![Command::CopyFrom(self.value()?)]);
    }
    
    impl_eq!(Expression);
}

impl Value for Deref {
    fn value(&self) -> Result<Reference, CompileError> {
        if let Some(t) = self.to_deref.downcast_ref::<Number>() {
            return Ok(Reference::Pointer(t.number));
        } else if let Some(t) = self.to_deref.downcast_ref::<Deref>() {
            if let Reference::Pointer(num) = t.value()? {
                return Ok(Reference::PointerPointer(num));
            }
        }
        
        Err(CompileError::NumberInsertionError)
    }
    
    impl_eq!(Value);
}

impl_partialeq!(Deref, to_deref);

#[derive(Debug)]
pub struct Add {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl Add {
    pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
        Self { left, right }
    }
}

impl Expression for Add {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(self.left.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![CopyTo(Reference::Pointer(add_addr))].into_iter())
            .chain(self.right.to_command(label_counter, add_addr)?)
            .chain(vec![Command::Add(Reference::Pointer(add_addr))].into_iter())
            .collect())
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(Add, left, right);

#[derive(Debug)]
pub struct Subtract {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl Subtract {
    pub fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
        Self { left, right }
    }
}

impl Expression for Subtract {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(self.left.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![CopyTo(Reference::Pointer(add_addr))])
            .chain(self.right.to_command(label_counter, add_addr)?)
            .chain(vec![Command::Subtract(Reference::Pointer(add_addr))])
            .collect())
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(Subtract, left, right);

#[derive(Debug, PartialEq, Eq)]
pub struct Loop {
    contents: Vec<Box<dyn Expression>>,
}

impl Loop {
    pub fn new(contents: Vec<Box<dyn Expression<>>>) -> Self {
        Self { contents }
    }
}

impl Expression for Loop {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        let top_label = Label::new(label_counter);
        let top_label_ref = LabelRef::new(&top_label);
        let (ok_contents, err_contents): (Vec<Result<Vec<Command>, CompileError>>, Vec<Result<Vec<Command>, CompileError>>) =
            self.contents.iter()
            .map(|e| e.to_command(label_counter, add_addr))
            .partition(|e| e.is_ok());
        for err_content in err_contents {
            return err_content;
        }
        Ok(vec![Command::Label(top_label)].into_iter()
            .chain(ok_contents.into_iter()
                .map(|e| e.unwrap())
                .flatten())
            .chain(vec![Command::Jump(top_label_ref)])
            .collect())
    }
    
    impl_eq!(Expression);
}

#[derive(Debug)]
pub struct Assign {
    left: Box<dyn Value>,
    right: Box<dyn Expression>,
}

impl Assign {
    pub fn new(left: Box<dyn Value>, right: Box<dyn Expression>) -> Self {
        Self { left, right }
    }
}

impl Expression for Assign {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(self.right.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::CopyTo(self.left.value()?)].into_iter())
            .collect())
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(Assign, left, right);

#[derive(Debug)]
pub struct If {
    condition: Box<dyn Logical>,
    to_run: Vec<Box<dyn Expression>>,
}

impl If {
    pub fn new(condition: Box<dyn Logical>, to_run: Vec<Box<dyn Expression>>) -> Self {
        Self { condition, to_run }
    }
}

impl Expression for If {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        let end_true_label = Label::new(label_counter);
        let ref_to = end_true_label.reference();
        
        let (ok, err): (Vec<Result<Vec<Command>, CompileError>>, Vec<Result<Vec<Command>, CompileError>>) = self.to_run.iter()
            .map(|e| e.to_command(label_counter, add_addr))
            .partition(|e| e.is_ok());
    
        for e in err {
            return e;
        }
        
        Ok(self.condition.to_commands(ref_to, label_counter, add_addr)?.into_iter()
            .chain(ok.into_iter()
                .map(|e| e.unwrap())
                .flatten())
            .chain(vec![Command::Label(end_true_label)])
            .collect())
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(If, condition, to_run);

#[derive(Debug)]
pub struct IfElse {
    condition: Box<dyn Logical>,
    if_true: Vec<Box<dyn Expression>>,
    if_false: Vec<Box<dyn Expression>>,
}

impl IfElse {
    pub fn new(condition: Box<dyn Logical>, if_true: Vec<Box<dyn Expression>>, if_false: Vec<Box<dyn Expression>>) -> Self {
        Self { condition, if_true, if_false }
    }
}

impl Expression for IfElse {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        let first_label = Label::new(label_counter);
        let second_label = Label::new(label_counter);
    
        let (ok_true, err): (Vec<Result<Vec<Command>, CompileError>>, Vec<Result<Vec<Command>, CompileError>>) = self.if_true.iter()
            .map(|e| e.to_command(label_counter, add_addr))
            .partition(|e| e.is_ok());
    
        for e in err {
            return e;
        }
    
        let (ok_false, err): (Vec<Result<Vec<Command>, CompileError>>, Vec<Result<Vec<Command>, CompileError>>) = self.if_false.iter()
            .map(|e| e.to_command(label_counter, add_addr))
            .partition(|e| e.is_ok());
    
        for e in err {
            return e;
        }
        
        Ok(self.condition.to_commands(first_label.reference(), label_counter, add_addr)?.into_iter()
            .chain(ok_true.into_iter()
                .map(|e| e.unwrap())
                .flatten())
            .chain(vec![Command::Jump(second_label.reference()), Command::Label(first_label)])
            .chain(ok_false.into_iter()
                .map(|e| e.unwrap())
                .flatten())
            .chain(vec![Command::Label(second_label)])
            .collect())
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(IfElse, condition, if_true, if_false);

#[derive(Debug)]
pub struct IsZero {
    expression: Box<dyn Expression>,
}

impl IsZero {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Logical for IsZero {
    fn to_commands(&self, end_label: LabelRef, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        let new_label = Label::new(label_counter);
        Ok(self.expression.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::JumpIfZero(new_label.reference()), Command::Jump(end_label), Command::Label(new_label)])
            .collect())
    }
    
    impl_eq!(Logical);
}

impl_partialeq!(IsZero, expression);

#[derive(Debug)]
pub struct NotZero {
    expression: Box<dyn Expression>,
}

impl NotZero {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Logical for NotZero {
    fn to_commands(&self, end_label: LabelRef, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(self.expression.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::JumpIfZero(end_label)])
            .collect())
    }
    
    impl_eq!(Logical);
}

impl_partialeq!(NotZero, expression);

#[derive(Debug)]
pub struct GreaterThanZero {
    expression: Box<dyn Expression>,
}

impl GreaterThanZero {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Logical for GreaterThanZero {
    fn to_commands(&self, end_label: LabelRef, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(self.expression.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::JumpIfZero(end_label.clone()), Command::JumpIfNegative(end_label)])
            .collect())
    }
    
    impl_eq!(Logical);
}

impl_partialeq!(GreaterThanZero, expression);

#[derive(Debug)]
pub struct LessThanZero {
    expression: Box<dyn Expression>,
}

impl LessThanZero {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Logical for LessThanZero {
    fn to_commands(&self, end_label: LabelRef, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        let label = Label::new(label_counter);
        Ok(self.expression.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::JumpIfNegative(label.reference()), Command::Jump(end_label), Command::Label(label)])
            .collect())
    }
    
    impl_eq!(Logical);
}

impl_partialeq!(LessThanZero, expression);

#[derive(Debug)]
pub struct GreaterOrEqualToZero {
    expression: Box<dyn Expression>,
}

impl GreaterOrEqualToZero {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Logical for GreaterOrEqualToZero {
    fn to_commands(&self, end_label: LabelRef, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(self.expression.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::JumpIfNegative(end_label)])
            .collect())
    }
    
    impl_eq!(Logical);
}

impl_partialeq!(GreaterOrEqualToZero, expression);

#[derive(Debug)]
pub struct LessOrEqualToZero {
    expression: Box<dyn Expression>,
}

impl LessOrEqualToZero {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

impl Logical for LessOrEqualToZero {
    fn to_commands(&self, end_label: LabelRef, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        let label = Label::new(label_counter);
        Ok(self.expression.to_command(label_counter, add_addr)?.into_iter()
            .chain(vec![Command::JumpIfNegative(label.reference()),
                        Command::JumpIfZero(label.reference()),
                        Command::Jump(end_label),
                        Command::Label(label)])
            .collect())
    }
    
    impl_eq!(Logical);
}

impl_partialeq!(LessOrEqualToZero, expression);

#[derive(Debug)]
pub struct Increment {
    to_increment: Box<dyn Value>,
}

impl Increment {
    pub fn new(to_increment: Box<dyn Value>) -> Self {
        Self { to_increment }
    }
}

impl Expression for Increment {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(vec![Command::Increment(self.to_increment.value()?)])
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(Increment, to_increment);

#[derive(Debug)]
pub struct Decrement {
    to_decrement: Box<dyn Value>,
}

impl Decrement {
    pub fn new(to_decrement: Box<dyn Value>) -> Self {
        Self { to_decrement }
    }
}

impl Expression for Decrement {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Ok(vec![Command::Decrement(self.to_decrement.value()?)])
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(Decrement, to_decrement);

#[derive(Debug)]
pub struct While {
    condition: Box<dyn Logical>,
    contents: Vec<Box<dyn Expression>>,
}

impl While {
    pub fn new(condition: Box<dyn Logical>, contents: Vec<Box<dyn Expression>>) -> Self {
        Self { condition, contents }
    }
}

impl Expression for While {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        let top_label = Label::new(label_counter);
        let top_label_ref = top_label.reference();
        let bottom_label = Label::new(label_counter);
    
        let (ok, err): (Vec<Result<Vec<Command>, CompileError>>, Vec<Result<Vec<Command>, CompileError>>) = self.contents.iter()
            .map(|e| e.to_command(label_counter, add_addr))
            .partition(|e| e.is_ok());
    
        for e in err {
            return e;
        }
        
        Ok(vec![Command::Label(top_label)].into_iter()
            .chain(self.condition.to_commands(bottom_label.reference(), label_counter, add_addr)?)
            .chain(ok.into_iter()
                .map(|e| e.unwrap())
                .flatten())
            .chain(vec![Command::Jump(top_label_ref), Command::Label(bottom_label)])
            .collect())
    }
    
    impl_eq!(Expression);
}

impl_partialeq!(While, condition, contents);

#[derive(Debug, PartialEq, Eq)]
pub struct Break {}

impl Expression for Break {
    fn to_command(&self, label_counter: &mut u8, add_addr: u8) -> Result<Vec<Command>, CompileError> {
        Err(CompileError::Error)
    }
    
    impl_eq!(Expression);
}