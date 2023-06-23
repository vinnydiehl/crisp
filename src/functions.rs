use crate::{error::CrispError, expr::{CrispExpr, FromCrispExpr, IntoCrispExpr}};

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

// Boolean operators

pub fn eq(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    let first_value = extract_value::<f64>(
        args.first().ok_or(CrispError::Reason("Expected 1+ arguments.".to_string()))?
    )?;

    // Fold across the list, comparing each value to the first
    list_foldl::<bool, f64>(&args[1..], true, |acc, n| acc && n == first_value)
}

macro_rules! fold_compare {
    ($f:expr) => {{
        |args: &[CrispExpr]| ->  Result<CrispExpr, CrispError> {
            let mut prev_value = extract_value::<f64>(
                args.first().ok_or(CrispError::Reason("Expected 1+ arguments.".to_string()))?
            )?;

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

fn list_foldl<T, U>(list: &[CrispExpr], init: T,
                    mut operation: impl FnMut(T, U) -> T) -> Result<CrispExpr, CrispError>
where
    T: IntoCrispExpr,
    U: FromCrispExpr + Copy
{
    let extracted_list = extract_list::<U>(list)?;

    if extracted_list.len() < 1 {
        return Err(CrispError::Reason("Expected 1+ arguments.".to_string()));
    };

    Ok(T::into_crisp_expr(
        extracted_list.iter().fold(init, |acc: T, &n: &U| operation(acc, n))
    ))
}

fn list_foldl1<T>(list: &[CrispExpr],
                  mut operation: impl FnMut(T, T) -> T) -> Result<CrispExpr, CrispError>
where
    T: FromCrispExpr + IntoCrispExpr + Copy
{
    let numbers = extract_list::<T>(list)?;

    if let Some((first, rest)) = numbers.split_first() {
        let result = rest.iter().fold(*first, |acc: T, &n: &T| operation(acc, n));
        Ok(T::into_crisp_expr(result))
    } else {
        Err(CrispError::Reason("Expected 1+ arguments.".to_string()))
    }
}

fn extract_list<T>(list: &[CrispExpr]) -> Result<Vec<T>, CrispError>
where
    T: FromCrispExpr,
{
    list.iter().map(|expr| extract_value::<T>(expr)).collect()
}

fn extract_value<T>(expr: &CrispExpr) -> Result<T, CrispError>
where
    T: FromCrispExpr,
{
    T::from_crisp_expr(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_list() {
        // Test case with valid numbers
        let list = vec![
            CrispExpr::Number(1.0),
            CrispExpr::Number(2.0),
            CrispExpr::Number(3.0),
        ];
        let result = extract_list::<f64>(&list);
        assert_eq!(result.unwrap(), vec![1.0, 2.0, 3.0]);

        // Test case with mixed types
        let list = vec![
            CrispExpr::Number(1.0),
            CrispExpr::Number(2.0),
            CrispExpr::Symbol("foo".to_string())
        ];
        let result = extract_list::<f64>(&list);
        assert!(matches!(result, Err(CrispError::Reason(_))));

        // Test bools
        let list = vec![
            CrispExpr::Bool(true),
            CrispExpr::Bool(false)
        ];
        let result = extract_list::<bool>(&list);
        assert_eq!(result.unwrap(), vec![true, false]);
    }

    #[test]
    fn test_extract_value() {
        // Test case with a valid number
        let expr = CrispExpr::Number(42.0);
        let result = extract_value::<f64>(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.0);

        // Test case with a non-number
        let expr = CrispExpr::Symbol("abc".to_string());
        let result = extract_value::<f64>(&expr);
        assert!(matches!(result, Err(CrispError::Reason(_))));
    }
}
