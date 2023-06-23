use crate::{error::CrispError, expr::CrispExpr, env::CrispEnv};

pub fn eval(expr: &CrispExpr, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    match expr {
        // It's a symbol, check the environment for it
        CrispExpr::Symbol(key) => env.data.get(key)
            .ok_or_else(|| CrispError::Reason(format!("Unexpected symbol: {}", key)))
            .map(|v| v.clone()),

        // It's a number or a bool, send it
        CrispExpr::Number(_) => Ok(expr.clone()),
        CrispExpr::Bool(_) => Ok(expr.clone()),

        // It's a list; the first node needs to be a function, the rest are args
        CrispExpr::List(list) => {
            let head = list.first()
                .ok_or_else(|| CrispError::Reason("Received an empty list.".to_string()))?;
            let args = &list[1..];

            match eval_keyword(head, args, env) {
                Some(response) => response,
                None => {
                    let first_eval = eval(head, env)?;
                    match first_eval {
                        CrispExpr::Func(f) => {
                            let args_eval: Result<Vec<_>, _> = args.iter()
                                                                   .map(|a| eval(a, env))
                                                                   .collect();
                            f(&args_eval?)
                        },
                        _ => Err(CrispError::Reason("List must begin with a function.".to_string()))
                    }
                }
            }
        },

        // Sorry, no infix functions yet :(
        CrispExpr::Func(_) =>
            Err(CrispError::Reason("Found unexpected function in list.".to_string()))
    }
}

fn eval_keyword(expr: &CrispExpr, args: &[CrispExpr],
                env: &mut CrispEnv) -> Option<Result<CrispExpr, CrispError>> {
    match expr {
        CrispExpr::Symbol(s) => {
            match s.as_ref() {
                "if" => Some(eval_if(args, env)),
                _ => None
            }
        },
        _ => None
    }
}

fn eval_if(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let predicate = args.first().ok_or_else(|| CrispError::Reason("No predicate found.".to_string()))?;
    let predicate_result = eval(predicate, env)?;

    match predicate_result {
        CrispExpr::Bool(b) => {
            // The function is going to be called like:
            //     (if (> a b) true_routine false_routine)
            // Depending on whether or not the predicate is true, we want to index
            // the args differently (0 is the predicate)
            let response = args.get(if b { 1 } else { 2 }).ok_or_else(||
                CrispError::Reason(format!("Predicate returned {} but nothing to evaluate.", b))
            )?;

            eval(response, env)
        },
        _ => Err(CrispError::Reason(format!("Unexpected predicate: `{}`", predicate)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{env::initialize_environment, expr::CrispExpr::*};

    macro_rules! list {
        ($($elem:expr),*) => {
            List(vec![$($elem),*])
        }
    }

    macro_rules! sym {
        ($name:expr) => {
            Symbol($name.to_string())
        }
    }

    #[test]
    fn test_eval_symbol_found() {
        let mut env = initialize_environment();
        env.data.insert("foo".to_string(), Number(42.0));

        let expr = sym!("foo");
        let result = eval(&expr, &mut env);

        assert_eq!(result, Ok(Number(42.0)));
    }

    #[test]
    fn test_eval_symbol_not_found() {
        let mut env = initialize_environment();

        let expr = sym!("x");
        assert!(eval(&expr, &mut env).is_err());
    }

    #[test]
    fn test_eval_number() {
        let mut env = initialize_environment();

        let expr = Number(3.14);
        let result = eval(&expr, &mut env);

        assert_eq!(result, Ok(Number(3.14)));
    }

    #[test]
    fn test_eval_list_empty() {
        let mut env = initialize_environment();

        let expr = list![];
        assert!(eval(&expr, &mut env).is_err());
    }

    #[test]
    fn test_eval_list_func() {
        let mut env = initialize_environment();

        let expr = list![
            sym!("+"),
            Number(2.0),
            Number(3.0),
            Number(4.0)
        ];
        let result = eval(&expr, &mut env);

        assert_eq!(result, Ok(Number(9.0)));
    }

    #[test]
    fn test_eval_nested_list() {
        let mut env = initialize_environment();

        let expr = list![
            sym!("*"),
            list![
                sym!("+"),
                Number(2.0),
                Number(3.0)
            ],
            Number(2.0),
            list![
                sym!("-"),
                Number(6.0),
                Number(2.0)
            ],
            Number(2.0)
        ];
        let result = eval(&expr, &mut env);

        assert_eq!(result, Ok(Number(80.0)));
    }

    #[test]
    fn test_eval_list_func_missing() {
        let mut env = initialize_environment();

        let expr = list![
            Number(2.0),
            Number(3.0),
            Number(4.0)
        ];
        assert!(eval(&expr, &mut env).is_err());
    }

    #[test]
    fn test_eval_list_func_mid() {
        let mut env = initialize_environment();

        let expr = list![
            Number(2.0),
            sym!("+"),
            Number(3.0)
        ];
        assert!(eval(&expr, &mut env).is_err());
    }

    // if keyword

    #[test]
    fn test_if_from_eval() {
        // Tests that the if keyword calls this routine. See the rest of the tests
        // in this section for more details.
        let list = list![sym!("if"), Bool(true), Number(1.0), Number(2.0)];
        assert_eq!(eval(&list, &mut initialize_environment()).unwrap(), Number(1.0));
    }

    #[test]
    fn test_if_result_selection() {
        let mut env = initialize_environment();

        // If true, it should select the first expression after the predicate
        let list = vec![
            Bool(true),
            Number(1.0),
            Number(2.0)
        ];
        assert_eq!(eval_if(&list, &mut env).unwrap(), Number(1.0));

        // If false, it should select the second expression after the predicate
        let list = vec![
            Bool(false),
            Number(1.0),
            Number(2.0)
        ];
        assert_eq!(eval_if(&list, &mut env).unwrap(), Number(2.0));
    }

    #[test]
    fn test_if_evaluation() {
        let mut env = initialize_environment();

        // If true, it should select the first expression after the predicate
        let list = vec![
            list![
                sym!("="),
                Number(5.0),
                Number(5.0)
            ],
            Number(1.0),
            Number(2.0)
        ];
        assert_eq!(eval_if(&list, &mut env).unwrap(), Number(1.0));

        // If false, it should select the second expression after the predicate
        let list = vec![
            list![
                sym!("="),
                Number(1.0),
                Number(5.0)
            ],
            Number(1.0),
            Number(2.0)
        ];
        assert_eq!(eval_if(&list, &mut env).unwrap(), Number(2.0));
    }

    #[test]
    fn test_if_result_evaluation() {
        // Results should be evaluated before they are returned

        let mut env = initialize_environment();

        // If true, it should select the first expression after the predicate
        let list = vec![
            list![
                sym!("="),
                Number(5.0),
                Number(5.0)
            ],
            list![
                sym!("+"),
                Number(3.0),
                Number(4.0)
            ],
            Number(2.0)
        ];
        assert_eq!(eval_if(&list, &mut env).unwrap(), Number(7.0));

        // If false, it should select the second expression after the predicate
        let list = vec![
            list![
                sym!("="),
                Number(1.0),
                Number(5.0)
            ],
            Number(1.0),
            list![
                sym!("+"),
                Number(3.0),
                Number(4.0)
            ]
        ];
        assert_eq!(eval_if(&list, &mut env).unwrap(), Number(7.0));
    }
}
