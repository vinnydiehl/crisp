use crate::{error::CrispError, expr::{CrispExpr, CrispLambda},
            env::{CrispEnv, env_get, env_new_for_lambda}, keywords::eval_keyword};

/// Evaluates an expression, resolving a node of the AST to a single value.
pub fn eval(expr: &CrispExpr, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    match expr {
        CrispExpr::List(list) if list.is_empty() => Ok(list![]),
        CrispExpr::List(list) => {
            match resolve(list, env) {
                Some(evaluated_expr) => Ok(evaluated_expr?),
                _ => Ok(CrispExpr::List(eval_across_list(&list, env)?))
            }
        },

        // It's a symbol, check the environment for it
        CrispExpr::Symbol(name) => env_get(name, env).ok_or_else(||
            parse_error_unwrapped!(format!("Could not find symbol: {}", name))
        ),

        CrispExpr::Char(_) => Ok(expr.clone()),
        CrispExpr::CrispString(_) => Ok(expr.clone()),
        CrispExpr::Nil => Ok(expr.clone()),
        CrispExpr::Number(_) => Ok(expr.clone()),
        CrispExpr::Bool(_) => Ok(expr.clone()),

        CrispExpr::Func(_) => parse_error!("Found unexpected function."),
        CrispExpr::Lambda(_) => parse_error!("Found unexpected lambda.")
    }
}

/// Given a slice of one or more [`CrispExpr`]s, this function will check the
/// first expression to see if it evalutates to a [`Func`](CrispExpr) or a
/// [`Lambda`](CrispExpr)- if so, it evaluates the entire slice as a
/// [`List`](CrispExpr) and returns the result. Otherwise returns `None`.
pub fn resolve(
    exprs: &[CrispExpr],
    env: &mut CrispEnv,
) -> Option<Result<CrispExpr, CrispError>> {
    let (head, tail) = exprs.split_first().unwrap();

    match head {
        CrispExpr::Symbol(_) => {
            match eval_keyword(head, tail, env) {
                Some(response) => Some(response),
                None => {
                    let evaluated_expr = eval(head, env);
                    match evaluated_expr {
                        Ok(CrispExpr::Func(func)) => Some(eval_func(func, tail, env)),
                        Ok(CrispExpr::Lambda(lambda)) => Some(eval_lambda(lambda, tail, env)),

                        Ok(_) if tail.is_empty() => Some(evaluated_expr),
                        Ok(_) => Some(join_and_eval_across_list(head, tail, env)),

                        Err(_) => Some(evaluated_expr)
                    }
                }
            }
        },

        CrispExpr::List(_) => {
            let result = match eval(head, env) {
                Ok(CrispExpr::Lambda(lambda)) => eval_lambda(lambda, tail, env),

                Ok(expr) if tail.is_empty() => Ok(expr),
                Ok(_) => {
                    let first_pass = join_and_eval_across_list(head, tail, env);
                    match first_pass {
                        Ok(CrispExpr::List(ref list)) => {
                            let (x, xs) = list.split_first().unwrap();
                            match eval(x, env) {
                                Ok(CrispExpr::Func(func)) => eval_func(func, xs, env),
                                Ok(CrispExpr::Lambda(lambda)) => eval_lambda(lambda, xs, env),
                                Err(e) => Err(e),
                                _ => first_pass
                            }
                        },

                        other => other
                    }
                }

                res => res
            };

            Some(result)
        },

        _ => None
    }
}

/// Takes a [`CrispExpr`] and a slice of `CrispExpr`s, joins them together and
/// calls [`eval_across_list`].
pub fn join_and_eval_across_list(
    head: &CrispExpr,
    tail: &[CrispExpr],
    env: &mut CrispEnv,
) -> Result<CrispExpr, CrispError> {
    match eval_across_list(&tail, env)? {
        mut eval_result => {
            eval_result.insert(0, head.clone());
            Ok(CrispExpr::List(eval_result))
        }
    }
}

/// Iterates across a slice of expressions, [`eval()`]ing each one.
pub fn eval_across_list(
    args: &[CrispExpr],
    env: &mut CrispEnv
) -> Result<Vec<CrispExpr>, CrispError> {
    args.iter().map(|a| eval(a, env)).collect()
}

/// Executes a built-in function `func` with the given `args`, returning
/// the result.
pub fn eval_func(
    func: fn(&[CrispExpr], &mut CrispEnv) -> Result<CrispExpr, CrispError>,
    args: &[CrispExpr],
    env: &mut CrispEnv
) -> Result<CrispExpr, CrispError> {
    func(&eval_across_list(args, env)?, env)
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
