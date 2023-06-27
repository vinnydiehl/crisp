use crate::{error::CrispError, expr::{CrispExpr, FromCrispExpr, IntoCrispExpr},
            env::CrispEnv, eval::eval_lambda};

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

pub fn crisp_puts(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let value = crisp_format(args, env)?;
    println!("{}", value);

    Ok(value)
}

pub fn crisp_print(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let value = crisp_format(args, env)?;
    print!("{}", value);

    Ok(value)
}

// Math operators

macro_rules! fold_operator {
    ($name:ident, $op:tt) => {
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

// Boolean operators

pub fn crisp_eq(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, -1);

    let first_value = extract_value::<f64>(args.first().unwrap())?;

    // Fold across the list, comparing each value to the first (as opposed to the
    // rest of the boolean comparisons, which compare to the previous value)
    backend_foldl::<bool, f64>(&args[1..], true, |acc, n| acc && n == first_value)
}

macro_rules! fold_compare {
    ($name:ident, $op:tt) => {
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

pub fn crisp_map(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, 2);

    match args.first().unwrap() {
        CrispExpr::Lambda(lambda) => {
            let n_args = match lambda.args.as_ref() {
                // The Symbol case will have already been handled when the list was
                // `eval_keyword_lambda()`ed into a CrispExpr, but we'll still print
                // it in the error since a Symbol is an acceptable input to a lambda
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

pub fn crisp_foldl(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 3, 3);

    match args.first().unwrap() {
        CrispExpr::Lambda(lambda) => {
            match lambda.args.as_ref() {
                CrispExpr::List(list) if list.len() != 2 =>
                    return standard_error!("Lambda for `foldl`/`foldl1` should take 2 arguments."),
                _ => {}
            };

            let mut acc = args.get(1).unwrap().clone();

            match args.get(2).unwrap() {
                CrispExpr::List(list) => {
                    for elem in list {
                        acc = eval_lambda(lambda.clone(), &vec![acc, elem.clone()], env)?.clone();
                    }

                    Ok(acc)
                },

                _ => type_error!("List")
            }
        },

        _ => type_error!("Lambda")
    }
}

pub fn crisp_foldl1(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, 2);

    // Plan of attack: pull out the first item in the list and construct a new set of
    // args into here which we will use to call `crisp_foldl()`.
    let mut new_args = vec![args.first().unwrap().clone()];

    match args.get(1).unwrap() {
        CrispExpr::List(list) => {
            match list.split_first() {
                Some((head, tail)) => {
                    new_args.push(head.clone());
                    new_args.push(CrispExpr::List(tail.to_vec()).clone());

                    crisp_foldl(&new_args[..], env)
                },

                None => standard_error!("List for `foldl1` is empty.")
            }
        },

        _ => type_error!("List")
    }
}

fn extract_value<T>(expr: &CrispExpr) -> Result<T, CrispError>
where T: FromCrispExpr {
    T::from_crisp_expr(expr)
}

fn extract_list<T>(list: &[CrispExpr]) -> Result<Vec<T>, CrispError>
where T: FromCrispExpr {
    list.iter().map(|expr| extract_value::<T>(expr)).collect()
}

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
    use std::rc::Rc;
    use crate::{expr::{CrispExpr::*, CrispLambda}, env::initialize_environment, eval::eval};

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

    // Boolean

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
            num_list![2.0, 3.0, 4.0]
        ];

        assert_eq!(crisp_map(&args, &mut env).unwrap(),
                   num_list![4.0, 6.0, 8.0]);

        let args = vec![
            lambda![
                args: ["a", "b"],
                func: [
                    sym!("+"),
                    sym!("a"),
                    sym!("b")
                ]
            ],
            num_list![1.0, 2.0, 10.0, 20.0, 100.0, 200.0]
        ];

        assert_eq!(crisp_map(&args, &mut env).unwrap(),
                   num_list![3.0, 30.0, 300.0]);

        // Test case passing in a function name
        env.data.insert("double".to_string(), lambda![
            args: ["a"],
            func: [
                sym!("*"),
                sym!("a"),
                Number(2.0)
            ]
        ]);

        // Needs to be eval'ed to turn the Symbol into a Lambda
        let result = eval(&list![
            sym!("map"),
            sym!("double"),
            num_list![2.0, 3.0, 4.0]
        ], &mut env).unwrap();

        assert_eq!(result, num_list![4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_foldl() {
        let mut env = initialize_environment();

        let args = vec![
            lambda![
                args: ["acc", "n"],
                func: [
                    sym!("+"),
                    sym!("acc"),
                    sym!("n")
                ]
            ],
            Number(10.0),
            num_list![1.0, 2.0, 3.0]
        ];
        let result = crisp_foldl(&args, &mut env).unwrap();

        assert_eq!(result, Number(16.0));
    }

    #[test]
    fn test_foldl1() {
        let mut env = initialize_environment();

        let args = vec![
            lambda![
                args: ["acc", "n"],
                func: [
                    sym!("+"),
                    sym!("acc"),
                    sym!("n")
                ]
            ],
            num_list![1.0, 2.0, 3.0]
        ];
        let result = crisp_foldl1(&args, &mut env).unwrap();

        assert_eq!(result, Number(6.0));
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
