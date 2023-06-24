use std::{fmt, rc::Rc};

use crate::error::{CrispError, type_error};

#[derive(Clone)]
pub enum CrispExpr {
    Symbol(String),
    Number(f64),
    Bool(bool),
    List(Vec<CrispExpr>),
    Func(fn(&[CrispExpr]) -> Result<CrispExpr, CrispError>),
    Lambda(CrispLambda)
}

#[derive(Clone)]
pub struct CrispLambda {
    pub args: Rc<CrispExpr>,
    pub func: Rc<CrispExpr>
}

impl PartialEq for CrispExpr {
    fn eq(&self, other: &Self) -> bool {
        // Implement your own equality logic here
        match (self, other) {
            (CrispExpr::Symbol(s1), CrispExpr::Symbol(s2)) => s1 == s2,
            (CrispExpr::Number(n1), CrispExpr::Number(n2)) => n1 == n2,
            (CrispExpr::List(l1), CrispExpr::List(l2)) => l1 == l2,
            (CrispExpr::Bool(b1), CrispExpr::Bool(b2)) => b1 == b2,
            _ => false
        }
    }
}

impl fmt::Debug for CrispExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for CrispExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            CrispExpr::Symbol(s) => s.clone(),
            CrispExpr::Number(n) => n.to_string(),
            CrispExpr::Bool(b) => b.to_string(),
            CrispExpr::List(list) => format!("({})",
                list.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(",")
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

impl FromCrispExpr for f64 {
    fn from_crisp_expr(expr: &CrispExpr) -> Result<Self, CrispError> {
        match expr {
            CrispExpr::Number(n) => Ok(*n),
            _ => type_error!("Number"),
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

impl IntoCrispExpr for bool {
    fn into_crisp_expr(self) -> CrispExpr {
        CrispExpr::Bool(self)
    }
}

impl IntoCrispExpr for f64 {
    fn into_crisp_expr(self) -> CrispExpr {
        CrispExpr::Number(self)
    }
}
