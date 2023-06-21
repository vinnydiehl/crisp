use std::fmt;

use crate::error::CrispError;

#[derive(Clone)]
pub enum CrispExpr {
    Symbol(String),
    Number(f64),
    List(Vec<CrispExpr>),
    Func(fn(&[CrispExpr]) -> Result<CrispExpr, CrispError>)
}

impl PartialEq for CrispExpr {
    fn eq(&self, other: &Self) -> bool {
        // Implement your own equality logic here
        match (self, other) {
            (CrispExpr::Symbol(s1), CrispExpr::Symbol(s2)) => s1 == s2,
            (CrispExpr::Number(n1), CrispExpr::Number(n2)) => n1 == n2,
            (CrispExpr::List(l1), CrispExpr::List(l2)) => l1 == l2,
            // Functions are not comparable
            (CrispExpr::Func(_), CrispExpr::Func(_)) => false,
            _ => false
        }
    }
}

impl fmt::Debug for CrispExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            CrispExpr::Symbol(s) => s.clone(),
            CrispExpr::Number(n) => n.to_string(),
            CrispExpr::List(list) => format!("({})",
                list.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(",")
            ),
            CrispExpr::Func(_) => "Function {}".to_string()
        };

        write!(f, "{}", str)
    }
}

impl fmt::Display for CrispExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            CrispExpr::Symbol(s) => s.clone(),
            CrispExpr::Number(n) => n.to_string(),
            CrispExpr::List(list) => format!("({})",
                list.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(",")
            ),
            CrispExpr::Func(_) => "Function {}".to_string()
        };

        write!(f, "{}", str)
    }
}
