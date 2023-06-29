use std::collections::HashSet;

use crate::{error::CrispError, expr::CrispExpr,
            env::CrispEnv, functions::{backend_foldl, extract_value}};

/// The `=` operator checks if all elements of a [`List`](CrispExpr)
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

    let uniq_values: Vec<&CrispExpr> = args.iter().collect::<HashSet<_>>().into_iter().collect();

    Ok(CrispExpr::Bool(uniq_values.len() == 1))
}

/// The `!=` operator checks if all elements of a [`List`](CrispExpr)
/// are unique.
///
/// # Examples
///
/// ```lisp
/// (!= 5 5)                ; => false
/// (!= 5 (+ 3 2) (- 10 5)) ; => false
/// (!= 2 5 4 5)            ; => false
/// (!= 5 1 4 0)            ; => true
/// ```
pub fn crisp_not_eq(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, -1);

    let uniq_values: Vec<&CrispExpr> = args.iter().collect::<HashSet<_>>().into_iter().collect();

    Ok(CrispExpr::Bool(args.len() == uniq_values.len()))
}

/// The numeric comparison operators check if a [`List`](CrispExpr) of
/// [`Number`](CrispExpr)s increases or decreases monotonically. These
/// functions are set with macros:
///
///  * `>`
///  * `>=`
///  * `<`
///  * `<=`
///
/// There are also some boolean comparison operators set through this macro:
///
///  * `&&`
///  * `||`
///
/// # Examples
///
/// ### Numeric comparisons
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
///
/// ### Boolean comparisons
///
/// `&&` is the logical AND operator, and `||` is for logical OR.
///
/// ```lisp
/// (&& (> 5 4) (= 3 3))                   ; => true
/// (&& (> 5 4) (= 3 9))                   ; => false
/// (&& (> 5 4) (= 3 3) (< 0 10) (>= 6 6)) ; => true
/// (&& (= 5 4) (= 3 3) (< 0 10) (>= 6 6)) ; => false
///
/// (|| (> 5 4) (= 3 9))                   ; => true
/// (|| (> 4 5) (= 3 9))                   ; => false
/// (|| (= 10 3) (= 4 6) (= 1 2) (> 5 4))  ;=> true
/// ````
macro_rules! fold_compare {
    ($name:ident, $op:tt, $type:ty) => {
        /// See [`fold_compare`].
        pub fn $name(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
            check_argument_error!(args, 2, -1);

            let mut prev_value = extract_value::<$type>(args.first().unwrap())?;

            backend_foldl::<bool, $type>(&args[1..], true, |acc, n| {
                let result = acc && prev_value $op n;
                prev_value = n;
                result
            })
        }
    };
}

fold_compare!(crisp_gt, >, f64);
fold_compare!(crisp_gte, >=, f64);
fold_compare!(crisp_lt, <, f64);
fold_compare!(crisp_lte, <=, f64);

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

fold_compare!(crisp_and, &&, bool);
fold_compare!(crisp_or, ||, bool);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{env::initialize_environment, expr::CrispExpr::*};

    #[test]
    fn test_eq() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_eq(&num_vec![5.0, 5.0], &mut env));
        crisp_assert!(crisp_eq(&num_vec![30.0, 30.0, 30.0], &mut env));
        crisp_assert!(crisp_eq(&string_vec!["foo", "foo"], &mut env));

        crisp_assert_false!(crisp_eq(&num_vec![5.0, 4.0], &mut env));
        crisp_assert_false!(crisp_eq(&num_vec![5.0, 4.0, 5.0], &mut env));
        crisp_assert_false!(crisp_eq(&string_vec!["foo", "bar"], &mut env));
        crisp_assert_false!(crisp_eq(&vec![str!("foo"), Number(5.0)], &mut env));
    }

    #[test]
    fn test_not_eq() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_not_eq(&num_vec![5.0, 4.0], &mut env));
        crisp_assert!(crisp_not_eq(&num_vec![5.0, 4.0, 10.0, 0.0], &mut env));
        crisp_assert!(crisp_not_eq(&string_vec!["foo", "bar"], &mut env));
        crisp_assert!(crisp_not_eq(&vec![str!("foo"), Number(5.0)], &mut env));

        crisp_assert_false!(crisp_not_eq(&num_vec![5.0, 5.0], &mut env));
        crisp_assert_false!(crisp_not_eq(&num_vec![5.0, 4.0, 10.0, 4.0], &mut env));
        crisp_assert_false!(crisp_not_eq(&string_vec!["foo", "foo"], &mut env));
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

        crisp_assert!(crisp_not(&bool_vec![false], &mut env));
        crisp_assert_false!(crisp_not(&bool_vec![true], &mut env));

        assert_eq!(crisp_not(&bool_vec![false, true, false], &mut env).unwrap(),
                   bool_list![true, false, true]);
    }

    #[test]
    fn test_and() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_and(&bool_vec![true, true], &mut env));
        crisp_assert!(crisp_and(&bool_vec![true, true, true], &mut env));
        crisp_assert!(crisp_and(&bool_vec![true, true, true, true, true], &mut env));

        crisp_assert_false!(crisp_and(&bool_vec![false, false], &mut env));
        crisp_assert_false!(crisp_and(&bool_vec![false, true], &mut env));
        crisp_assert_false!(crisp_and(&bool_vec![true, true, false, true], &mut env));
        crisp_assert_false!(crisp_and(&bool_vec![true, true, true, true, false], &mut env));
    }

    #[test]
    fn test_or() {
        let mut env = initialize_environment();

        crisp_assert!(crisp_or(&bool_vec![false, true], &mut env));
        crisp_assert!(crisp_or(&bool_vec![true, false, true], &mut env));
        crisp_assert!(crisp_or(&bool_vec![false, false, false, false, false, true], &mut env));

        crisp_assert_false!(crisp_or(&bool_vec![false, false], &mut env));
        crisp_assert_false!(crisp_or(&bool_vec![false, false, false, false], &mut env));
    }
}
