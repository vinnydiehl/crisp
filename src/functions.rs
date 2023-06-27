mod boolean;
mod io;
mod lists;
mod math;

pub use boolean::*;
pub use io::*;
pub use lists::*;
pub use math::*;

use crate::{error::CrispError, expr::{CrispExpr, FromCrispExpr, IntoCrispExpr}};

/// Extracts a value from a `CrispExpr`.
fn extract_value<T>(expr: &CrispExpr) -> Result<T, CrispError>
where T: FromCrispExpr {
    T::from_crisp_expr(expr)
}

/// Maps across a List, extracting all of the values of the `CrispExpr`s
/// into a `Vec`.
fn extract_list<T>(list: &[CrispExpr]) -> Result<Vec<T>, CrispError>
where T: FromCrispExpr {
    list.iter().map(|expr| extract_value::<T>(expr)).collect()
}

/// For internal use with Rust functions. See `crisp_foldl()` for the crisp
/// `foldl` function.
fn backend_foldl<T, U>(args: &[CrispExpr], init: T,
                       mut operation: impl FnMut(T, U) -> T) -> Result<CrispExpr, CrispError>
where
    T: IntoCrispExpr,
    U: FromCrispExpr + Copy
{
    check_argument_error!(args, 1, -1);

    Ok(T::into_crisp_expr(
        extract_list::<U>(args)?.iter().fold(init, |acc: T, &n: &U| operation(acc, n))
    ))
}

/// For internal use with Rust functions. See `crisp_foldl1()` for the crisp
/// `foldl1` function.
fn backend_foldl1<T>(args: &[CrispExpr],
                     mut operation: impl FnMut(T, T) -> T) -> Result<CrispExpr, CrispError>
where T: FromCrispExpr + IntoCrispExpr + Copy {
    check_argument_error!(args, 2, -1);

    let numbers = extract_list::<T>(args)?;
    let (first, rest) = numbers.split_first().unwrap();

    Ok(T::into_crisp_expr(
        rest.iter().fold(*first, |acc: T, &n: &T| operation(acc, n))
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::CrispExpr::*;

    #[test]
    fn test_extract_value() {
        // Test case with a valid number
        let expr = Number(42.0);
        let result = extract_value::<f64>(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.0);

        // Test case with a non-number
        let expr = sym!("abc");
        let result = extract_value::<f64>(&expr);
        assert!(matches!(result, Err(CrispError::TypeError(_))));
    }

    #[test]
    fn test_extract_list() {
        // Mixed types should error
        let result = extract_list::<f64>(&vec![
            Number(1.0),
            Number(2.0),
            sym!("foo")
        ]);
        assert!(matches!(result, Err(CrispError::TypeError(_))));

        // Numbers
        let result = extract_list::<f64>(&num_vec![1.0, 2.0, 3.0]);
        assert_eq!(result.unwrap(), vec![1.0, 2.0, 3.0]);

        // Bools
        let result = extract_list::<bool>(&vec![Bool(true), Bool(false)]);
        assert_eq!(result.unwrap(), vec![true, false]);
    }

    #[test]
    fn test_backend_foldl() {
        let list = num_vec![3.0, 4.0, 2.0];
        assert_eq!(backend_foldl::<f64, f64>(&list, 8.0, |acc, n| acc - n).unwrap(),
                   Number(-1.0));
    }

    #[test]
    fn test_backend_foldl1() {
        let list = num_vec![3.0, 4.0, 2.0];
        assert_eq!(backend_foldl1::<f64>(&list, |acc, n| acc * n).unwrap(),
                   Number(24.0));
    }
}
