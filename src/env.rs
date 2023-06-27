use std::{collections::HashMap, rc::Rc};

use crate::{error::{CrispError, argument_error, parse_error}, expr::CrispExpr, functions};

#[derive(Clone)]
pub struct CrispEnv<'a> {
    pub data: HashMap<String, CrispExpr>,
    pub parent: Option<&'a CrispEnv<'a>>
}

pub fn initialize_environment<'a>() -> CrispEnv<'a> {
    let mut data: HashMap<String, CrispExpr> = HashMap::new();

    macro_rules! add_function {
        ($name:expr, $rust_function:ident) => {
            data.insert($name.to_string(), CrispExpr::Func(functions::$rust_function));
        }
    }

    add_function!("format", crisp_format);
    add_function!("puts", crisp_puts);
    add_function!("print", crisp_print);

    add_function!("+", crisp_add);
    add_function!("-", crisp_sub);
    add_function!("*", crisp_mult);
    add_function!("/", crisp_div);
    add_function!("mod", crisp_mod);

    add_function!("=", crisp_eq);
    add_function!(">", crisp_gt);
    add_function!(">=", crisp_gte);
    add_function!("<", crisp_lt);
    add_function!("<=", crisp_lte);

    add_function!("map", crisp_map);

    CrispEnv { data, parent: None }
}

/// Searches for a key `name` within the scope `env` or any outer scope
/// outside of that.
pub fn env_get(name: &str, env: &CrispEnv) -> Option<CrispExpr> {
    match env.data.get(name) {
        Some(expr) => Some(expr.clone()),
        None => {
            match &env.parent {
                Some(parent) => env_get(name, &parent),
                None => None
            }
        }
    }
}

/// When a lambda is called, this routine is called, creating a new scope.
///
/// # Arguments
///
///  * `lambda_args`: `CrispExpr::List` of `CrispExpr::Symbols` containing
///                   the names of the arguments.
///  * `arg_passed_exprs`: The unevaluated expressions that were passed into
///                        the lambda when it was called.
///  * `parent_env`: The scope just outside the lambda.
///
/// # Returns
///
/// The `CrispEnv` for this scope, or a `CrispError` if there were any problems.
pub fn env_new_for_lambda<'a>(
    lambda_args: Rc<CrispExpr>,
    arg_passed_exprs: &[CrispExpr],
    parent_env: &'a mut CrispEnv
) -> Result<CrispEnv<'a>, CrispError> {
    let arg_names = parse_symbol_list(lambda_args)?;

    let n_args: i32 = arg_names.len().try_into().unwrap_or_else(|_| i32::MAX);
    if n_args != arg_passed_exprs.len().try_into().unwrap_or_else(|_| i32::MAX) {
        return argument_error!(n_args, n_args);
    };

    // Insert the inputs to the arguments into the `env.data` for this scope
    let mut data: HashMap<String, CrispExpr> = HashMap::new();
    for (name, value) in arg_names.iter().zip(arg_passed_exprs.iter()) {
        data.insert(name.clone(), value.clone());
    }

    Ok(CrispEnv { data, parent: Some(parent_env) })
}

/// Given a reference counted pointer to a `CrispExpr::List` full of
/// `CrispExpr::Symbol`s, processes it into a `Vec<String>`.
fn parse_symbol_list(list: Rc<CrispExpr>) -> Result<Vec<String>, CrispError> {
    let arg_names = match list.as_ref() {
        CrispExpr::List(list) => Ok(list.clone()),
        _ => parse_error!("Lambda expected a list of arguments.")
    }?;

    arg_names.iter().map(|arg| {
        match arg {
            CrispExpr::Symbol(name) => Ok(name.clone()),
            _ => parse_error!("Lambda expected symbols in the argument list.")
        }
    }).collect()
}
