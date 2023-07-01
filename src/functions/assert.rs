use std::process;

use crate::{env::CrispEnv, error::CrispError, expr::CrispExpr};

use super::crisp_eq;

const FAIL_ERR_CODE: i32 = 101;

/// `assert` takes a predicate and returns `true` if the predicate evaluates
/// to `true`, otherwise it terminates the program with an error code.
///
/// # Examples
///
/// ```lisp
/// assert (> 5 4)
/// assert (= 3 4)  ; this would terminate the program
/// assert (< 1 10)
/// ```
pub fn crisp_assert(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 1, 1);

    match args.first().unwrap() {
        CrispExpr::Bool(b) => {
            if !b {
                process::exit(FAIL_ERR_CODE);
            }

            Ok(CrispExpr::Bool(true))
        },

        _ => type_error!("Bool")
    }
}

/// `assert_false` takes a predicate and returns `true` if the predicate evaluates
/// to `false`, otherwise it terminates the program with an error code.
///
/// # Examples
///
/// ```lisp
/// assert_false (< 5 4)
/// assert_false (= 4 4)  ; this would terminate the program
/// assert_false (> 1 10)
/// ```
pub fn crisp_assert_false(
    args: &[CrispExpr],
    _env: &mut CrispEnv
) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 1, 1);

    match args.first().unwrap() {
        CrispExpr::Bool(b) => {
            if *b {
                process::exit(FAIL_ERR_CODE);
            }

            Ok(CrispExpr::Bool(true))
        },

        _ => type_error!("Bool")
    }
}

/// `assert_eq` returns `true` if all aruments are equal, otherwise it
/// terminates the program with an error code.
///
/// # Examples
///
/// ```lisp
/// assert_eq 5 5
/// assert_eq 5 4   ; this would terminate the program
/// assert_eq 10 10
/// ```
pub fn crisp_assert_eq(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    match crisp_eq(args, env)? {
        CrispExpr::Bool(b) => {
            if !b {
                process::exit(FAIL_ERR_CODE);
            }

            Ok(CrispExpr::Bool(true))
        },

        _ => type_error!("Bool")
    }
}

/// `assert_not_eq` returns `true` if all aruments are equal, otherwise it
/// terminates the program with an error code.
///
/// # Examples
///
/// ```lisp
/// assert_not_eq 5 4
/// assert_not_eq 5 5   ; this would terminate the program
/// assert_not_eq 10 4
/// ```
pub fn crisp_assert_not_eq(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    match crisp_eq(args, env)? {
        CrispExpr::Bool(b) => {
            if b {
                process::exit(FAIL_ERR_CODE);
            }

            Ok(CrispExpr::Bool(true))
        },

        _ => type_error!("Bool")
    }
}
