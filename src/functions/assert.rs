use std::process;

use crate::{env::CrispEnv, error::CrispError, expr::CrispExpr};

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
                process::exit(101);
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
                process::exit(101);
            }

            Ok(CrispExpr::Bool(true))
        },

        _ => type_error!("Bool")
    }
}
