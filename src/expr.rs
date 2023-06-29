use std::{fmt, hash::{Hasher, Hash}, rc::Rc};

use crate::{env::CrispEnv, error::CrispError, escape_string};

#[derive(Clone)]
pub enum CrispExpr {
    Symbol(String),
    CrispString(String),
    Number(f64),
    Bool(bool),
    List(Vec<CrispExpr>),
    Func(fn(&[CrispExpr], &mut CrispEnv) -> Result<CrispExpr, CrispError>),
    Lambda(CrispLambda)
}

#[derive(Clone)]
pub struct CrispLambda {
    pub args: Rc<CrispExpr>,
    pub func: Rc<CrispExpr>
}

impl PartialEq for CrispExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CrispExpr::Symbol(s1), CrispExpr::Symbol(s2)) => s1 == s2,
            (CrispExpr::CrispString(s1), CrispExpr::CrispString(s2)) => s1 == s2,
            (CrispExpr::Number(n1), CrispExpr::Number(n2)) => n1 == n2,
            (CrispExpr::List(l1), CrispExpr::List(l2)) => l1 == l2,
            (CrispExpr::Bool(b1), CrispExpr::Bool(b2)) => b1 == b2,
            _ => false
        }
    }
}

impl Eq for CrispExpr {}

impl fmt::Debug for CrispExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for CrispExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            CrispExpr::Symbol(s) => s.clone(),
            CrispExpr::CrispString(s) => s.clone(),
            CrispExpr::Number(n) => n.to_string(),
            CrispExpr::Bool(b) => b.to_string(),
            CrispExpr::List(list) => format!("({})",
                list.iter().map(|e| {
                    match e {
                        CrispExpr::CrispString(s) => escape_string(s),
                        _ => e.to_string()
                    }
                }).collect::<Vec<String>>().join(" ")
            ),
            CrispExpr::Func(_) => "<Func>".to_string(),
            CrispExpr::Lambda(_) => "<Lambda>".to_string()
        };

        write!(f, "{}", str)
    }
}

pub trait FromCrispExpr: Sized {
    fn from_crisp_expr(expr: &CrispExpr) -> Result<Self, CrispError>;
}

impl FromCrispExpr for String {
    fn from_crisp_expr(expr: &CrispExpr) -> Result<Self, CrispError> {
        match expr {
            CrispExpr::CrispString(s) => Ok(s.clone()),
            _ => type_error!("String"),
        }
    }
}

impl FromCrispExpr for f64 {
    fn from_crisp_expr(expr: &CrispExpr) -> Result<Self, CrispError> {
        match expr {
            CrispExpr::Number(n) => Ok(*n),
            _ => type_error!("Number"),
        }
    }
}

impl FromCrispExpr for Vec<CrispExpr> {
    fn from_crisp_expr(expr: &CrispExpr) -> Result<Self, CrispError> {
        match expr {
            CrispExpr::List(list) => Ok(list.clone()),
            _ => type_error!("List"),
        }
    }
}

impl FromCrispExpr for bool {
    fn from_crisp_expr(expr: &CrispExpr) -> Result<Self, CrispError> {
        match expr {
            CrispExpr::Bool(b) => Ok(*b),
            _ => type_error!("Bool"),
        }
    }
}

pub trait IntoCrispExpr {
    fn into_crisp_expr(self) -> CrispExpr;
}

impl IntoCrispExpr for String {
    fn into_crisp_expr(self) -> CrispExpr {
        CrispExpr::CrispString(self)
    }
}

impl IntoCrispExpr for f64 {
    fn into_crisp_expr(self) -> CrispExpr {
        CrispExpr::Number(self)
    }
}

impl IntoCrispExpr for Vec<CrispExpr> {
    fn into_crisp_expr(self) -> CrispExpr {
        CrispExpr::List(self)
    }
}

impl IntoCrispExpr for bool {
    fn into_crisp_expr(self) -> CrispExpr {
        CrispExpr::Bool(self)
    }
}

impl Hash for CrispExpr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CrispExpr::Symbol(s) => s.hash(state),
            CrispExpr::CrispString(s) => s.hash(state),
            // Convert the number to its IEEE 754 binary representation and hash it
            CrispExpr::Number(n) => state.write_u64(n.to_bits()),
            // Convert the boolean to a u8 (0 for false, 1 for true)
            CrispExpr::Bool(b) => state.write_u8(*b as u8),
            CrispExpr::List(list) => list.hash(state),
            // TODO: Figure out a way to hash lambdas/funcs
            _ => {}
        }
    }
}

impl Hash for CrispLambda {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the Rc references to ensure the content is hashed
        self.args.as_ref().hash(state)
    }
}
