use crate::{error::CrispError, expr::{CrispExpr, CrispLambda},
            env::{CrispEnv, env_get, env_new_for_lambda}, keywords::eval_keyword};

/// Evaluates an expression, resolving a node of the AST to a single value.
pub fn eval(expr: &CrispExpr, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    match expr {
        CrispExpr::List(list) if list.is_empty() => Ok(list![]),
        CrispExpr::List(list) => {
            let head = list.first().unwrap();
            let args = &list[1..];

            match eval_keyword(head, args, env) {
                // Is the first item in the list a keyword?
                Some(response) => response,
                None => {
                    // Is the first item a function? (Func is built-in, Lambda is user-defined)
                    match eval(head, env)? {
                        CrispExpr::Func(func) => func(&eval_across_list(args, env)?, env),
                        CrispExpr::Lambda(lambda) => eval_lambda(lambda, args, env),

                        // None of the above, evaluate everything and send it
                        first => {
                            let mut eval_result = eval_across_list(&list[1..], env)?;
                            eval_result.insert(0, first);
                            Ok(CrispExpr::List(eval_result))
                        }
                    }
                }
            }
        },

        // It's a symbol, check the environment for it
        CrispExpr::Symbol(name) => env_get(name, env).ok_or_else(||
            parse_error_unwrapped!(format!("Could not find symbol: {}", name))
        ),

        CrispExpr::Char(_) => Ok(expr.clone()),
        CrispExpr::CrispString(_) => Ok(expr.clone()),
        CrispExpr::Number(_) => Ok(expr.clone()),
        CrispExpr::Bool(_) => Ok(expr.clone()),

        CrispExpr::Func(_) => parse_error!("Found unexpected function."),
        CrispExpr::Lambda(_) => parse_error!("Found unexpected lambda.")
    }
}

/// Iterates across a slice of expressions, [`eval()`]ing each one.
pub fn eval_across_list(
    args: &[CrispExpr],
    env: &mut CrispEnv
) -> Result<Vec<CrispExpr>, CrispError> {
    args.iter().map(|a| eval(a, env)).collect()
}

/// Calls a [`Lambda`](CrispExpr) with the arguments given in `args`, returns
/// the return value of that `Lambda` call.
pub fn eval_lambda(
    lambda: CrispLambda,
    args: &[CrispExpr],
    env: &mut CrispEnv
) -> Result<CrispExpr, CrispError> {
    eval(&lambda.func,
         &mut env_new_for_lambda(lambda.args, &eval_across_list(args, env)?, env)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{env::initialize_environment, expr::CrispExpr::*};

    #[test]
    fn test_eval_symbol_found() {
        let mut env = initialize_environment();
        env.data.insert("foo".to_string(), Number(42.0));

        let expr = sym!("foo");
        let result = eval(&expr, &mut env).unwrap();

        assert_eq!(result, Number(42.0));
    }

    #[test]
    fn test_eval_symbol_not_found() {
        let mut env = initialize_environment();
        crisp_assert_err!(eval(&sym!("x"), &mut env), ParseError);
    }

    #[test]
    fn test_eval_number() {
        let mut env = initialize_environment();

        let expr = Number(3.14);
        let result = eval(&expr, &mut env).unwrap();

        assert_eq!(result, Number(3.14));
    }

    #[test]
    fn test_eval_list_empty() {
        let mut env = initialize_environment();

        assert_eq!(eval(&list![], &mut env),
                   Ok(list![]));
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
        let result = eval(&expr, &mut env).unwrap();

        assert_eq!(result, Number(9.0));
    }

    #[test]
    fn test_eval_list_nested_func() {
        let mut env = initialize_environment();

        env.data.insert("n".to_string(), Number(5.0));
        let expr = list![list![list![
            sym!("let"),
            sym!("n"),
            list![
                sym!("+"),
                Number(1.0),
                sym!("n")
            ]
        ]]];
        eval(&expr, &mut env).unwrap();

        assert_eq!(env.data.get("n").unwrap(), &Number(6.0));
    }

    #[test]
    fn test_eval_list_no_func() {
        let mut env = initialize_environment();

        let expr = list![
            Number(2.0),
            Number(3.0),
            Number(4.0)
        ];
        let result = eval(&expr, &mut env).unwrap();

        assert_eq!(result, expr);

        // Should evaluate nested items
        let expr = list![
            Number(2.0),
            Number(3.0),
            list![
                sym!("+"),
                Number(4.0),
                Number(5.0)
            ]
        ];
        let result = eval(&expr, &mut env).unwrap();
        let expected = list![
            Number(2.0),
            Number(3.0),
            Number(9.0)
        ];

        assert_eq!(result, expected);
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
        let result = eval(&expr, &mut env).unwrap();

        assert_eq!(result, Number(80.0));
    }
}
