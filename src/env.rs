use std::collections::HashMap;

use crate::{error::CrispError, expr::CrispExpr};

#[derive(Clone)]
pub struct CrispEnv {
    pub data: HashMap<String, CrispExpr>
}

macro_rules! bool_compare {
    ($f:expr) => {{
        |args: &[CrispExpr]| -> Result<CrispExpr, CrispError> {
            let xs = extract_list_numbers(args)?;
            let head = xs.first().ok_or_else(|| CrispError::Reason("Expected 1+ arguments.".to_string()))?;
            let tail = &xs[1..];

            // Recursively consume the remainder of the list, ensuring a monotonic order
            // beginning-to-end e.g. (> 10 5 2 1) or (<= 2 5 5 7 8)
            fn f(head: &f64, tail: &[f64]) -> bool {
                match tail.first() {
                    Some(next) => $f(head, next) && f(next, &tail[1..]),
                    None => true
                }
            }

            Ok(CrispExpr::Bool(f(head, tail)))
        }
    }};
}

pub fn initialize_environment() -> CrispEnv {
    let mut data: HashMap<String, CrispExpr> = HashMap::new();

    data.insert(
        "+".to_string(),
        CrispExpr::Func(|args: &[CrispExpr]| -> Result<CrispExpr, CrispError> {
            list_foldl(args, |acc, n| acc + n)
        })
    );

    data.insert(
        "-".to_string(),
        CrispExpr::Func(|args: &[CrispExpr]| -> Result<CrispExpr, CrispError> {
            list_foldl(args, |acc, n| acc - n)
        })
    );

    data.insert(
        "*".to_string(),
        CrispExpr::Func(|args: &[CrispExpr]| -> Result<CrispExpr, CrispError> {
            list_foldl(args, |acc, n| acc * n)
        })
    );

    data.insert(
        "/".to_string(),
        CrispExpr::Func(|args: &[CrispExpr]| -> Result<CrispExpr, CrispError> {
            list_foldl(args, |acc, n| acc / n)
        })
    );

    // Boolean shit

    data.insert(
        "=".to_string(),
        CrispExpr::Func(bool_compare!(|a, b| a == b))
    );

    data.insert(
        ">".to_string(),
        CrispExpr::Func(bool_compare!(|a, b| a > b))
    );

    data.insert(
        "<".to_string(),
        CrispExpr::Func(bool_compare!(|a, b| a < b))
    );

    data.insert(
        ">=".to_string(),
        CrispExpr::Func(bool_compare!(|a, b| a >= b))
    );

    data.insert(
        "<=".to_string(),
        CrispExpr::Func(bool_compare!(|a, b| a <= b))
    );

    CrispEnv { data }
}

fn list_foldl(list: &[CrispExpr],
              operation: impl Fn(f64, f64) -> f64) -> Result<CrispExpr, CrispError> {
    let numbers = extract_list_numbers(list)?;

    if let Some((first, rest)) = numbers.split_first() {
        let result = rest.iter().fold(*first, |acc, &n| operation(acc, n));
        Ok(CrispExpr::Number(result))
    } else {
        Err(CrispError::Reason("Expected 1+ arguments.".to_string()))
    }
}

fn extract_list_numbers(list: &[CrispExpr]) -> Result<Vec<f64>, CrispError> {
    list.iter().map(extract_number).collect()
}

fn extract_number(expr: &CrispExpr) -> Result<f64, CrispError> {
    match expr {
        CrispExpr::Number(n) => Ok(*n),
        _ => Err(CrispError::Reason("Expected a number.".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_list_numbers() {
        // Test case with valid numbers
        let list = vec![
            CrispExpr::Number(1.0),
            CrispExpr::Number(2.0),
            CrispExpr::Number(3.0),
        ];
        let result = extract_list_numbers(&list);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1.0, 2.0, 3.0]);

        // Test case with a non-number (at the end of the previous list)
        let list = vec![
            CrispExpr::Number(1.0),
            CrispExpr::Number(2.0),
            CrispExpr::Symbol("foo".to_string())
        ];
        let result = extract_list_numbers(&list);
        assert!(matches!(result, Err(CrispError::Reason(_))));
    }

    #[test]
    fn test_extract_number() {
        // Test case with a valid number
        let expr = CrispExpr::Number(42.0);
        let result = extract_number(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.0);

        // Test case with a non-number
        let expr = CrispExpr::Symbol("abc".to_string());
        let result = extract_number(&expr);
        assert!(matches!(result, Err(CrispError::Reason(_))));
    }
}
