use crate::{error::{CrispError, parse_error, parse_error_unwrapped}, expr::CrispExpr,
            env::{CrispEnv, env_get, env_new_for_lambda}, keywords::eval_keyword};

pub fn eval(expr: &CrispExpr, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    match expr {
        // It's a symbol, check the environment for it
        CrispExpr::Symbol(name) => env_get(name, env).ok_or_else(||
            parse_error_unwrapped!(format!("Could not find symbol: {}", name))
        ),

        // It's a string, number, or bool, send it
        CrispExpr::CrispString(_) => Ok(expr.clone()),
        CrispExpr::Number(_) => Ok(expr.clone()),
        CrispExpr::Bool(_) => Ok(expr.clone()),

        // It's a list; the first node needs to be a function, the rest are args
        CrispExpr::List(list) => {
            let head = list.first()
                .ok_or_else(|| parse_error_unwrapped!("Received an empty list."))?;
            let args = &list[1..];

            match eval_keyword(head, args, env) {
                Some(response) => response,
                None => {
                    let first_eval = eval(head, env)?;
                    match first_eval {
                        CrispExpr::Func(func) => func(&eval_across_list(args, env)?),

                        CrispExpr::Lambda(lambda) => {
                            eval(
                                &lambda.func,
                                &mut env_new_for_lambda(lambda.args, args, env)?
                            )
                        },

                        _ => parse_error!("List must begin with a function.")
                    }
                }
            }
        },

        // Sorry, no infix functions yet :(
        CrispExpr::Func(_) => parse_error!("Found unexpected function in list."),
        CrispExpr::Lambda(_) => parse_error!("Found unexpected lambda in list.")
    }
}

pub fn eval_across_list(
    args: &[CrispExpr],
    env: &mut CrispEnv
) -> Result<Vec<CrispExpr>, CrispError> {
    args.iter().map(|a| eval(a, env)).collect()
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
