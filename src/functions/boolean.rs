use crate::{error::CrispError, expr::CrispExpr,
            env::CrispEnv, functions::{backend_foldl, extract_value}};

/// The equality operator ensures that all elements of a [`List`](CrispExpr)
/// are the same.
///
/// # Examples
///
/// ```lisp
/// (= 5 5)                ; => true
/// (= 5 (+ 3 2) (- 10 5)) ; => true
/// (= 5 5 4 5)            ; => false
/// ```
pub fn crisp_eq(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, -1);

    let first_value = extract_value::<f64>(args.first().unwrap())?;

    // Fold across the list, comparing each value to the first (as opposed to the
    // rest of the boolean comparisons, which compare to the previous value)
    backend_foldl::<bool, f64>(&args[1..], true, |acc, n| acc && n == first_value)
}

/// The comparison operators ensure that a [`List`](CrispExpr) increases or
/// decreases monotonically. These functions are set with macros:
///
///  * `>`
///  * `>=`
///  * `<`
///  * `<=`
///
/// # Examples
///
/// ```lisp
/// (> 5 4)      ; => true
/// (> 5 4 3 1)  ; => true
/// (> 5 4 4 1)  ; => false
/// (>= 5 4 4 1) ; => true
/// (> 3 10)     ; => false
/// (< 3 10)     ; => true
/// (<= 3 3)     ; => true
/// ```
macro_rules! fold_compare {
    ($name:ident, $op:tt) => {
        /// See [`fold_compare`].
        pub fn $name(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
            check_argument_error!(args, 2, -1);

            let mut prev_value = extract_value::<f64>(args.first().unwrap())?;

            backend_foldl::<bool, f64>(&args[1..], true, |acc, n| {
                let result = acc && prev_value $op n;
                prev_value = n;
                result
            })
        }
    };
}

fold_compare!(crisp_gt, >);
fold_compare!(crisp_gte, >=);
fold_compare!(crisp_lt, <);
fold_compare!(crisp_lte, <=);

/// The `!` operator inverts one or more [`Bool`](CrispExpr)s. If one argument
/// is provided, a `Bool` will be returned, otherwise the results will be
/// mapped into a [`List`](CrispExpr) of `Bool`s.
///
/// # Examples
///
/// ```lisp
/// ! true         ; => false
/// ! false true   ; => (true false)
/// ! (= 3 3) true ; => (false false)
/// ```
pub fn crisp_not(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 1, -1);

    args.iter()
        .map(|elem| match elem {
            CrispExpr::Bool(b) => Ok(CrispExpr::Bool(!b)),
            _ => type_error!("Bool"),
        })
        .collect::<Result<Vec<CrispExpr>, CrispError>>()
        .map(|list| {
            if list.len() == 1 {
                list.into_iter().next().unwrap()
            } else {
                CrispExpr::List(list)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CrispExpr::*, env::initialize_environment};

    #[test]
    fn test_eq() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_eq(&num_vec![5.0, 5.0], &mut env));
        crisp_assert!(crisp_eq(&num_vec![30.0, 30.0, 30.0], &mut env));

        crisp_assert_false!(crisp_eq(&num_vec![5.0, 4.0], &mut env));
        crisp_assert_false!(crisp_eq(&num_vec![5.0, 4.0, 5.0], &mut env));
    }

    #[test]
    fn test_gt() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_gt(&num_vec![5.0, 4.0], &mut env));
        crisp_assert!(crisp_gt(&num_vec![4.0, 2.0, 0.0], &mut env));

        crisp_assert_false!(crisp_gt(&num_vec![5.0, 6.0], &mut env));
        crisp_assert_false!(crisp_gt(&num_vec![4.0, 2.0, 2.0], &mut env));
    }

    #[test]
    fn test_gte() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_gte(&num_vec![5.0, 4.0], &mut env));
        crisp_assert!(crisp_gte(&num_vec![4.0, 2.0, 0.0], &mut env));
        crisp_assert!(crisp_gte(&num_vec![4.0, 2.0, 2.0, 1.5], &mut env));

        crisp_assert_false!(crisp_gte(&num_vec![5.0, 6.0], &mut env));
        crisp_assert_false!(crisp_gte(&num_vec![5.0, 4.0, 2.5, 3.0], &mut env));
    }

    #[test]
    fn test_lt() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_lt(&num_vec![5.0, 6.0], &mut env));
        crisp_assert!(crisp_lt(&num_vec![4.0, 7.0, 10.0], &mut env));

        crisp_assert_false!(crisp_lt(&num_vec![5.0, 1.0], &mut env));
        crisp_assert_false!(crisp_lt(&num_vec![4.0, 5.0, 5.0], &mut env));
    }

    #[test]
    fn test_lte() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_lte(&num_vec![5.0, 6.0], &mut env));
        crisp_assert!(crisp_lte(&num_vec![4.0, 7.0, 10.0], &mut env));
        crisp_assert!(crisp_lte(&num_vec![4.0, 5.0, 5.0, 5.5], &mut env));

        crisp_assert_false!(crisp_lte(&num_vec![5.0, 1.0], &mut env));
        crisp_assert_false!(crisp_lte(&num_vec![5.0, 7.0, 8.0, 7.5], &mut env));
    }

    #[test]
    fn test_not() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_not(&vec![Bool(false)], &mut env));
        crisp_assert_false!(crisp_not(&vec![Bool(true)], &mut env));

        assert_eq!(crisp_not(&vec![Bool(false), Bool(true), Bool(false)], &mut env).unwrap(),
                   list![Bool(true), Bool(false), Bool(true)]);
    }
}
