use std::collections::HashMap;

use crate::{expr::CrispExpr, functions};

#[derive(Clone)]
pub struct CrispEnv {
    pub data: HashMap<String, CrispExpr>
}

pub fn initialize_environment() -> CrispEnv {
    let mut data: HashMap<String, CrispExpr> = HashMap::new();

    macro_rules! add_function {
        ($name:expr, $rust_function:ident) => {
            data.insert($name.to_string(), CrispExpr::Func(functions::$rust_function));
        }
    }

    add_function!("+", add);
    add_function!("-", sub);
    add_function!("*", mult);
    add_function!("/", div);
    add_function!("mod", modulus);

    add_function!("=", eq);
    add_function!(">", gt);
    add_function!(">=", gte);
    add_function!("<", lt);
    add_function!("<=", lte);

    CrispEnv { data }
}
