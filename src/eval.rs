use crate::{error::CrispError, expr::CrispExpr, env::CrispEnv, keywords::eval_keyword};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{env::initialize_environment, expr::CrispExpr::*, macros::*};

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
}
