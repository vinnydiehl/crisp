use crate::{error::{CrispError, argument_error}, expr::{CrispExpr, FromCrispExpr, IntoCrispExpr}};

// Math operators

pub fn add(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc + n)
}

pub fn sub(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc - n)
}

pub fn mult (args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc * n)
}

pub fn div (args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc / n)
}

pub fn modulus (args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc % n)
}

// Boolean operators

pub fn eq(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    argument_error!(args, 2, -1);

    let first_value = extract_value::<f64>(args.first().unwrap())?;

    // Fold across the list, comparing each value to the first
    list_foldl::<bool, f64>(&args[1..], true, |acc, n| acc && n == first_value)
}

macro_rules! fold_compare {
    ($f:expr) => {{
        |args: &[CrispExpr]| ->  Result<CrispExpr, CrispError> {
            argument_error!(args, 2, -1);

            let mut prev_value = extract_value::<f64>(args.first().unwrap())?;

            list_foldl::<bool, f64>(&args[1..], true, |acc, n| {
                let result = acc && $f(n, prev_value);
                prev_value = n;
                result
            })
        }
    }};
}

pub fn gt(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    fold_compare!(|a, b| a < b)(args)
}

pub fn gte(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    fold_compare!(|a, b| a <= b)(args)
}

pub fn lt(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    fold_compare!(|a, b| a > b)(args)
}

pub fn lte(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    fold_compare!(|a, b| a >= b)(args)
}

fn extract_value<T>(expr: &CrispExpr) -> Result<T, CrispError>
where T: FromCrispExpr {
    T::from_crisp_expr(expr)
}

fn extract_list<T>(list: &[CrispExpr]) -> Result<Vec<T>, CrispError>
where T: FromCrispExpr {
    list.iter().map(|expr| extract_value::<T>(expr)).collect()
}

fn list_foldl<T, U>(args: &[CrispExpr], init: T,
                    mut operation: impl FnMut(T, U) -> T) -> Result<CrispExpr, CrispError>
where
    T: IntoCrispExpr,
    U: FromCrispExpr + Copy
{
    argument_error!(args, 1, -1);

    Ok(T::into_crisp_expr(
        extract_list::<U>(args)?.iter().fold(init, |acc: T, &n: &U| operation(acc, n))
    ))
}

fn list_foldl1<T>(args: &[CrispExpr],
                  mut operation: impl FnMut(T, T) -> T) -> Result<CrispExpr, CrispError>
where T: FromCrispExpr + IntoCrispExpr + Copy {
    argument_error!(args, 2, -1);

    let numbers = extract_list::<T>(args)?;
    let (first, rest) = numbers.split_first().unwrap();

    Ok(T::into_crisp_expr(
        rest.iter().fold(*first, |acc: T, &n: &T| operation(acc, n))
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expr::CrispExpr::*, macros::*};

    // Math

    #[test]
    fn test_add() {
        crisp_assert_eq!(add(&num_list![6.0, 9.0]), 15.0);
        crisp_assert_eq!(add(&num_list![1.0, 2.0, 3.0]), 6.0);
    }

    #[test]
    fn test_sub() {
        crisp_assert_eq!(sub(&num_list![6.0, 9.0]), -3.0);
        crisp_assert_eq!(sub(&num_list![1.0, 2.0, 3.0]), -4.0);
    }

    #[test]
    fn test_mult() {
        crisp_assert_eq!(mult(&num_list![6.0, 9.0]), 54.0);
        crisp_assert_eq!(mult(&num_list![5.0, 2.0, 3.0]), 30.0);
    }

    #[test]
    fn test_div() {
        crisp_assert_eq!(div(&num_list![9.0, 2.0]), 4.5);
        crisp_assert_eq!(div(&num_list![30.0, 3.0, 2.0]), 5.0);
    }

    #[test]
    fn test_mod() {
        crisp_assert_eq!(modulus(&num_list![9.0, 2.0]), 1.0);
        crisp_assert_eq!(modulus(&num_list![35.0, 25.0, 6.0]), 4.0);
    }

    // Boolean

    #[test]
    fn test_eq() {
        crisp_assert!(eq(&num_list![5.0, 5.0]));
        crisp_assert!(eq(&num_list![30.0, 30.0, 30.0]));

        crisp_assert_false!(eq(&num_list![5.0, 4.0]));
        crisp_assert_false!(eq(&num_list![5.0, 4.0, 5.0]));
    }

    #[test]
    fn test_gt() {
        crisp_assert!(gt(&num_list![5.0, 4.0]));
        crisp_assert!(gt(&num_list![4.0, 2.0, 0.0]));

        crisp_assert_false!(gt(&num_list![5.0, 6.0]));
        crisp_assert_false!(gt(&num_list![4.0, 2.0, 2.0]));
    }

    #[test]
    fn test_gte() {
        crisp_assert!(gte(&num_list![5.0, 4.0]));
        crisp_assert!(gte(&num_list![4.0, 2.0, 0.0]));
        crisp_assert!(gte(&num_list![4.0, 2.0, 2.0, 1.5]));

        crisp_assert_false!(gte(&num_list![5.0, 6.0]));
        crisp_assert_false!(gte(&num_list![5.0, 4.0, 2.5, 3.0]));
    }

    #[test]
    fn test_lt() {
        crisp_assert!(lt(&num_list![5.0, 6.0]));
        crisp_assert!(lt(&num_list![4.0, 7.0, 10.0]));

        crisp_assert_false!(lt(&num_list![5.0, 1.0]));
        crisp_assert_false!(lt(&num_list![4.0, 5.0, 5.0]));
    }

    #[test]
    fn test_lte() {
        crisp_assert!(lte(&num_list![5.0, 6.0]));
        crisp_assert!(lte(&num_list![4.0, 7.0, 10.0]));
        crisp_assert!(lte(&num_list![4.0, 5.0, 5.0, 5.5]));

        crisp_assert_false!(lte(&num_list![5.0, 1.0]));
        crisp_assert_false!(lte(&num_list![5.0, 7.0, 8.0, 7.5]));
    }

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
        let result = extract_list::<f64>(&num_list![1.0, 2.0, 3.0]);
        assert_eq!(result.unwrap(), vec![1.0, 2.0, 3.0]);

        // Bools
        let result = extract_list::<bool>(&vec![Bool(true), Bool(false)]);
        assert_eq!(result.unwrap(), vec![true, false]);
    }

    #[test]
    fn test_list_foldl() {
        let list = num_list![3.0, 4.0, 2.0];
        assert_eq!(list_foldl::<f64, f64>(&list, 8.0, |acc, n| acc - n).unwrap(),
                   Number(-1.0));
    }

    #[test]
    fn test_list_foldl1() {
        let list = num_list![3.0, 4.0, 2.0];
        assert_eq!(list_foldl1::<f64>(&list, |acc, n| acc * n).unwrap(),
                   Number(24.0));
    }
}
