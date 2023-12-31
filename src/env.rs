use std::{collections::HashMap, rc::Rc};

use crate::{error::CrispError, expr::CrispExpr, functions};

#[derive(Clone)]
pub struct CrispEnv<'a> {
    pub data: HashMap<String, CrispExpr>,
    pub parent: Option<&'a CrispEnv<'a>>
}

/// Initializes and returns an environment with all of the built-in functions.
/// This will be the top-level scope under which all other environments
/// will nest.
pub fn initialize_environment<'a>() -> CrispEnv<'a> {
    let mut data: HashMap<String, CrispExpr> = HashMap::new();

    macro_rules! add_function {
        ($name:expr, $rust_function:ident) => {
            data.insert($name.to_string(), CrispExpr::Func(functions::$rust_function));
        }
    }

    add_function!("assert", crisp_assert);
    add_function!("assert-false", crisp_assert_false);
    add_function!("assert-eq", crisp_assert_eq);
    add_function!("assert-not-eq", crisp_assert_not_eq);

    add_function!("format", crisp_format);
    add_function!("puts", crisp_puts);
    add_function!("print", crisp_print);

    add_function!("+", crisp_add);
    add_function!("-", crisp_sub);
    add_function!("*", crisp_mult);
    add_function!("/", crisp_div);
    add_function!("mod", crisp_mod);

    add_function!("=", crisp_eq);
    add_function!("!=", crisp_not_eq);
    add_function!(">", crisp_gt);
    add_function!(">=", crisp_gte);
    add_function!("<", crisp_lt);
    add_function!("<=", crisp_lte);
    add_function!("!", crisp_not);
    add_function!("&&", crisp_and);
    add_function!("||", crisp_or);

    add_function!("cons", crisp_cons);
    add_function!("map", crisp_map);
    add_function!("foldl", crisp_foldl);
    add_function!("foldl1", crisp_foldl1);

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

/// When a [`Lambda`](CrispExpr) is called, this routine is called, creating a
/// new scope.
///
/// # Arguments
///
///  * `lambda_args`: [`List`](CrispExpr) of [`Symbol`](CrispExpr)s containing
///                   the names of the arguments.
///  * `arg_passed_exprs`: The unevaluated expressions that were passed into
///                        the `Lambda` when it was called.
///  * `parent_env`: The scope just outside the `Lambda`.
///
/// # Returns
///
/// The [`CrispEnv`] for this scope, or a [`CrispError`] if there were any
/// problems.
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

/// Given a reference counted pointer to a [`List`](CrispExpr) full of
/// [`Symbol`](CrispExpr)s, processes it into a [`Vec<String>`].
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
