use crate::{error::{CrispError, argument_error, check_argument_error, type_error},
            expr::{CrispExpr, FromCrispExpr, IntoCrispExpr}, env::CrispEnv, eval::eval_lambda};

use dyn_fmt::AsStrFormatExt;

pub fn crisp_format(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    if let Some((format_str, format_args)) = args.split_first() {
        return Ok(str!(match format_args {
            [] => format!("{}", format_str),
            _ => format_str.to_string().format(format_args)
        }));
    }

    argument_error!(1, -1)
}

pub fn puts(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let value = crisp_format(args, env)?;
    println!("{}", value);

    Ok(value)
}

pub fn print(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let value = crisp_format(args, env)?;
    print!("{}", value);

    Ok(value)
}

// Math operators

pub fn add(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc + n)
}

pub fn sub(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc - n)
}

pub fn mult(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc * n)
}

pub fn div(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc / n)
}

pub fn modulus(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    list_foldl1::<f64>(args, |acc, n| acc % n)
}

// Boolean operators

pub fn eq(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, -1);

    let first_value = extract_value::<f64>(args.first().unwrap())?;

    // Fold across the list, comparing each value to the first
    list_foldl::<bool, f64>(&args[1..], true, |acc, n| acc && n == first_value)
}

macro_rules! fold_compare {
    ($f:expr) => {{
        |args: &[CrispExpr]| ->  Result<CrispExpr, CrispError> {
            check_argument_error!(args, 2, -1);

            let mut prev_value = extract_value::<f64>(args.first().unwrap())?;

            list_foldl::<bool, f64>(&args[1..], true, |acc, n| {
                let result = acc && $f(n, prev_value);
                prev_value = n;
                result
            })
        }
    }};
}

pub fn gt(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    fold_compare!(|a, b| a < b)(args)
}

pub fn gte(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    fold_compare!(|a, b| a <= b)(args)
}

pub fn lt(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    fold_compare!(|a, b| a > b)(args)
}

pub fn lte(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
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

pub fn crisp_map(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, 2);

    match args.first().unwrap() {
        CrispExpr::Lambda(lambda) => {
            let n_args = match lambda.args.as_ref() {
                CrispExpr::Symbol(_) => 1,
                CrispExpr::List(list) => list.len(),
                _ => return type_error!("Symbol || List<Symbol>")
            };

            match args.get(1).unwrap() {
                CrispExpr::List(list) => {
                    let mut result = Vec::new();
                    for chunk in list.chunks(n_args) {
                        result.push(eval_lambda(lambda.clone(), chunk, env)?);
                    }

                    Ok(CrispExpr::List(result))
                },

                _ => type_error!("List")
            }
        },

        _ => type_error!("Lambda")
    }
}

fn list_foldl<T, U>(args: &[CrispExpr], init: T,
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

fn list_foldl1<T>(args: &[CrispExpr],
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
    use std::rc::Rc;

    use super::*;
    use crate::{expr::{CrispExpr::*, CrispLambda}, env::initialize_environment};

    #[test]
    fn test_format() {
        let mut env = initialize_environment();

        let result = crisp_format(&vec![
            str!("test")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test"));

        let result = crisp_format(&vec![
            str!("test: {}"),
            str!("foo")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo"));

        let result = crisp_format(&vec![
            str!("test: {}"),
            Number(123.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: 123"));

        let result = crisp_format(&vec![
            str!("{}{}"),
            Number(1.0),
            Number(2.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("12"));

        let result = crisp_format(&vec![
            str!("{} a {} b {}"),
            str!("1"),
            Number(2.0),
            Bool(true),
        ], &mut env).unwrap();

        assert_eq!(result, str!("1 a 2 b true"));
    }

    #[test]
    fn test_format_escape() {
        let mut env = initialize_environment();

        let result = crisp_format(&vec![
            str!("{{}}"),
            Number(42.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("{}"));

        let result = crisp_format(&vec![
            str!("{}{{}}{}"),
            Number(24.0),
            Number(42.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("24{}42"));

        let result = crisp_format(&vec![
            str!("test {{ escaped braces }} {{:3}}"),
            Number(42.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("test { escaped braces } {:3}"));

        // With no arguments, braces don't need to be escaped
        let result = crisp_format(&vec![
            str!("test {{ escaped braces }} {{:3}}"),
        ], &mut env).unwrap();

        assert_eq!(result, str!("test {{ escaped braces }} {{:3}}"));
    }

    #[test]
    fn test_format_too_many_args() {
        let mut env = initialize_environment();

        // It should discard the superfluous args
        let result = crisp_format(&vec![
            str!("test: {}"),
            str!("foo"),
            str!("bar"),
            str!("baz")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo"));
    }

    #[test]
    fn test_format_too_few_args() {
        let mut env = initialize_environment();

        // It should fill in left-to-right and leave the remaining braces
        let result = crisp_format(&vec![
            str!("test: {} {} {}"),
            str!("foo")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo  "));

        let result = crisp_format(&vec![
            str!("test: {} {} {}"),
            str!("foo"),
            str!("bar")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo bar "));
    }

    // Math

    #[test]
    fn test_add() {
        let mut env = initialize_environment();

        crisp_assert_eq!(add(&num_list![6.0, 9.0], &mut env), 15.0);
        crisp_assert_eq!(add(&num_list![1.0, 2.0, 3.0], &mut env), 6.0);
    }

    #[test]
    fn test_sub() {
        let mut env = initialize_environment();

        crisp_assert_eq!(sub(&num_list![6.0, 9.0], &mut env), -3.0);
        crisp_assert_eq!(sub(&num_list![1.0, 2.0, 3.0], &mut env), -4.0);
    }

    #[test]
    fn test_mult() {
        let mut env = initialize_environment();

        crisp_assert_eq!(mult(&num_list![6.0, 9.0], &mut env), 54.0);
        crisp_assert_eq!(mult(&num_list![5.0, 2.0, 3.0], &mut env), 30.0);
    }

    #[test]
    fn test_div() {
        let mut env = initialize_environment();

        crisp_assert_eq!(div(&num_list![9.0, 2.0], &mut env), 4.5);
        crisp_assert_eq!(div(&num_list![30.0, 3.0, 2.0], &mut env), 5.0);
    }

    #[test]
    fn test_mod() {
        let mut env = initialize_environment();

        crisp_assert_eq!(modulus(&num_list![9.0, 2.0], &mut env), 1.0);
        crisp_assert_eq!(modulus(&num_list![35.0, 25.0, 6.0], &mut env), 4.0);
    }

    // Boolean

    #[test]
    fn test_eq() {
        let mut env = initialize_environment();

        crisp_assert!(eq(&num_list![5.0, 5.0], &mut env));
        crisp_assert!(eq(&num_list![30.0, 30.0, 30.0], &mut env));

        crisp_assert_false!(eq(&num_list![5.0, 4.0], &mut env));
        crisp_assert_false!(eq(&num_list![5.0, 4.0, 5.0], &mut env));
    }

    #[test]
    fn test_gt() {
        let mut env = initialize_environment();

        crisp_assert!(gt(&num_list![5.0, 4.0], &mut env));
        crisp_assert!(gt(&num_list![4.0, 2.0, 0.0], &mut env));

        crisp_assert_false!(gt(&num_list![5.0, 6.0], &mut env));
        crisp_assert_false!(gt(&num_list![4.0, 2.0, 2.0], &mut env));
    }

    #[test]
    fn test_gte() {
        let mut env = initialize_environment();

        crisp_assert!(gte(&num_list![5.0, 4.0], &mut env));
        crisp_assert!(gte(&num_list![4.0, 2.0, 0.0], &mut env));
        crisp_assert!(gte(&num_list![4.0, 2.0, 2.0, 1.5], &mut env));

        crisp_assert_false!(gte(&num_list![5.0, 6.0], &mut env));
        crisp_assert_false!(gte(&num_list![5.0, 4.0, 2.5, 3.0], &mut env));
    }

    #[test]
    fn test_lt() {
        let mut env = initialize_environment();

        crisp_assert!(lt(&num_list![5.0, 6.0], &mut env));
        crisp_assert!(lt(&num_list![4.0, 7.0, 10.0], &mut env));

        crisp_assert_false!(lt(&num_list![5.0, 1.0], &mut env));
        crisp_assert_false!(lt(&num_list![4.0, 5.0, 5.0], &mut env));
    }

    #[test]
    fn test_lte() {
        let mut env = initialize_environment();

        crisp_assert!(lte(&num_list![5.0, 6.0], &mut env));
        crisp_assert!(lte(&num_list![4.0, 7.0, 10.0], &mut env));
        crisp_assert!(lte(&num_list![4.0, 5.0, 5.0, 5.5], &mut env));

        crisp_assert_false!(lte(&num_list![5.0, 1.0], &mut env));
        crisp_assert_false!(lte(&num_list![5.0, 7.0, 8.0, 7.5], &mut env));
    }

    #[test]
    fn test_map() {
        let mut env = initialize_environment();

        let args = vec![
            lambda![
                args: ["a"],
                func: [
                    sym!("*"),
                    sym!("a"),
                    Number(2.0)
                ]
            ],
            List(num_list![2.0, 3.0, 4.0])
        ];

        assert_eq!(crisp_map(&args, &mut env).unwrap(),
                   List(num_list![4.0, 6.0, 8.0]));

        let args = vec![
            lambda![
                args: ["a", "b"],
                func: [
                    sym!("+"),
                    sym!("a"),
                    sym!("b")
                ]
            ],
            List(num_list![1.0, 2.0, 10.0, 20.0, 100.0, 200.0])
        ];

        assert_eq!(crisp_map(&args, &mut env).unwrap(),
                   List(num_list![3.0, 30.0, 300.0]));
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
