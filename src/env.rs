use std::collections::HashMap;

use crate::{expr::CrispExpr, functions};

#[derive(Clone)]
pub struct CrispEnv {
    pub data: HashMap<String, CrispExpr>
}

pub fn initialize_environment() -> CrispEnv {
    let mut data: HashMap<String, CrispExpr> = HashMap::new();

    data.insert("+".to_string(), CrispExpr::Func(functions::add));
    data.insert("-".to_string(), CrispExpr::Func(functions::sub));
    data.insert("*".to_string(), CrispExpr::Func(functions::mult));
    data.insert("/".to_string(), CrispExpr::Func(functions::div));

    data.insert("=".to_string(), CrispExpr::Func(functions::eq));
    data.insert(">".to_string(), CrispExpr::Func(functions::gt));
    data.insert(">=".to_string(), CrispExpr::Func(functions::gte));
    data.insert("<".to_string(), CrispExpr::Func(functions::lt));
    data.insert("<=".to_string(), CrispExpr::Func(functions::lte));

    CrispEnv { data }
}
