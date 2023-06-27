use crate::{error::CrispError, expr::CrispExpr, env::CrispEnv, eval::eval_lambda};

pub fn crisp_cons(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, 2);

    let first = args.first().unwrap();

    match args.get(1).unwrap() {
        CrispExpr::List(list) => {
            let mut new_list = list.clone();
            new_list.insert(0, first.clone());

            Ok(CrispExpr::List(new_list.clone()))
        },

        _ => type_error!("List")
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use crate::{expr::{CrispExpr::*, CrispLambda}, env::initialize_environment, eval::eval};

    #[test]
    fn test_cons() {
        let mut env = initialize_environment();

        let result = crisp_cons(&vec![
            str!("test:"),
            num_list!(4.0, 2.0)
        ], &mut env).unwrap();

        let expected = list![
            str!("test:"),
            Number(4.0),
            Number(2.0)
        ];

        assert_eq!(result, expected);

        let result = crisp_cons(&vec![
            num_list!(1.0, 2.0),
            num_list!(3.0, 4.0)
        ], &mut env).unwrap();

        let expected = list![
            num_list!(1.0, 2.0),
            Number(3.0),
            Number(4.0)
        ];

        assert_eq!(result, expected);
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
}
