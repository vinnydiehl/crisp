use crate::{error::CrispError, expr::CrispExpr, env::CrispEnv, functions::backend_foldl1};

/// The math operators fold across the [`List`](CrispExpr) from left-to-right,
/// applying the operator to the next element. The result is that `+` is more
/// of a `List` sum function than a simple addition function. The following
/// functions are set in this manner with macros:
///
///  * `+`: Addition
///  * `-`: Subtraction
///  * `*`: Multiplication
///  * `/`: Division
///  * `mod`: Modulus
///
/// # Examples
///
/// ```lisp
/// (+ 1 2 3) ; => 6
/// (- 3 2 1) ; => 0
/// (* 2 10)  ; => 20
/// (/ 9 2)   ; => 4.5
/// (mod 9 2) ; => 1
/// ```
macro_rules! fold_operator {
    ($name:ident, $op:tt) => {
        /// See [`fold_operator`].
        pub fn $name(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
            backend_foldl1::<f64>(args, |acc, n| acc $op n)
        }
    };
}

fold_operator!(crisp_add, +);
fold_operator!(crisp_sub, -);
fold_operator!(crisp_mult, *);
fold_operator!(crisp_div, /);
fold_operator!(crisp_mod, %);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::initialize_environment;

    #[test]
    fn test_add() {
        let mut env = initialize_environment();

        crisp_assert_eq!(crisp_add(&num_vec![6.0, 9.0], &mut env), 15.0);
        crisp_assert_eq!(crisp_add(&num_vec![1.0, 2.0, 3.0], &mut env), 6.0);
    }

    #[test]
    fn test_sub() {
        let mut env = initialize_environment();

        crisp_assert_eq!(crisp_sub(&num_vec![6.0, 9.0], &mut env), -3.0);
        crisp_assert_eq!(crisp_sub(&num_vec![1.0, 2.0, 3.0], &mut env), -4.0);
    }

    #[test]
    fn test_mult() {
        let mut env = initialize_environment();

        crisp_assert_eq!(crisp_mult(&num_vec![6.0, 9.0], &mut env), 54.0);
        crisp_assert_eq!(crisp_mult(&num_vec![5.0, 2.0, 3.0], &mut env), 30.0);
    }

    #[test]
    fn test_div() {
        let mut env = initialize_environment();

        crisp_assert_eq!(crisp_div(&num_vec![9.0, 2.0], &mut env), 4.5);
        crisp_assert_eq!(crisp_div(&num_vec![30.0, 3.0, 2.0], &mut env), 5.0);
    }

    #[test]
    fn test_mod() {
        let mut env = initialize_environment();

        crisp_assert_eq!(crisp_mod(&num_vec![9.0, 2.0], &mut env), 1.0);
        crisp_assert_eq!(crisp_mod(&num_vec![35.0, 25.0, 6.0], &mut env), 4.0);
    }
}
