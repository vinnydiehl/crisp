use std::{rc::Rc, process};

use crate::{error::CrispError, expr::{CrispExpr, CrispLambda}, env::CrispEnv, eval::eval};

/// When a [`Symbol`](CrispExpr) begins a [`List`](CrispExpr), it is passed
/// through this function which checks if it is a keyword and if so, evaluates
/// the list via one of the routines in this file.
pub fn eval_keyword(expr: &CrispExpr, args: &[CrispExpr],
                    env: &mut CrispEnv) -> Option<Result<CrispExpr, CrispError>> {
    match expr {
        CrispExpr::Symbol(s) => {
            match s.as_ref() {
                "if" => Some(eval_if(args, env)),
                "let" => Some(eval_let(args, env)),
                "\\" => Some(eval_keyword_lambda(args)),
                "fn" => Some(eval_fn(args, env)),
                "exit" => Some(eval_exit(args, env)),
                _ => None
            }
        },
        _ => None
    }
}

/// An `if` expression has the following syntax:
///
/// ```lisp
/// (if predicate true_expr false_expr)
/// ```
///
/// `predicate` is an expression which evaluates to `true` or `false`.
/// Depending on the result, the `if` expression will evaluate and
/// return `true_expr` or `false_expr`.
///
/// # Examples
///
/// ```lisp
/// (if (> 5 4) (+ 0 5) (- 0 5)) ; => 5
/// (if (< 5 4) (+ 0 5) (- 0 5)) ; => -5
/// ```
fn eval_if(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 3, 3);

    match eval(args.first().unwrap(), env)? {
        CrispExpr::Bool(b) => {
            // Depending on whether or not the predicate is true, we want to index
            // the args differently (0 is the predicate)
            let response = args.get(if b { 1 } else { 2 }).unwrap();

            eval(response, env)
        },
        _ => type_error!("Bool")
    }
}

/// `let` is the variable assignment keyword. It returns the assigned value.
///
/// # Usage
///
/// ```lisp
/// let var_name value
/// ```
///
/// # Examples
///
/// ```lisp
/// let str "foo"
/// let n 42
/// let xs (1 2 3 4 5)
/// ```
fn eval_let(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, 2);

    let name_sym = args.first().unwrap();
    let name = match name_sym {
        CrispExpr::Symbol(s) => Ok(s.clone()),
        _ => type_error!("Symbol")
    }?;

    let value = eval(args.get(1).unwrap(), env)?;
    env.data.insert(name, value.clone());

    Ok(value.clone())
}

/// A [`Lambda`](CrispExpr) is an anonymous function. It is declared like so:
///
/// ```lisp
/// (\ args expression)
/// ```
///
/// `args` is either a [`Symbol`](CrispExpr) or a [`List`](CrispExpr) of
/// `Symbol`s, and when the `Lambda` is called, the values given as arguments
/// will be available within the expression with those variable names.
///
/// # Examples
///
/// ```lisp
/// ((\ (a b) (* a b)) 3 5)       ; => 15
/// map (\ n (* 2 n)) (1 2 3 4 5) ; => (2 4 6 8 10)
/// ```
fn eval_keyword_lambda(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, 2);

    let a = args.first().unwrap().clone();
    let arg_list = match a {
        CrispExpr::List(_) => a,
        CrispExpr::Symbol(_) => list![a],
        _ => return type_error!("Symbol || List<Symbol>"),
    };

    Ok(CrispExpr::Lambda(CrispLambda {
        args: Rc::new(arg_list),
        func: Rc::new(args.get(1).unwrap().clone()),
    }))
}

/// `fn` defines a function by creating a [`Lambda`](CrispExpr) and saving it
/// to the environment. `fn` is syntactic sugar for:
///
/// ```lisp
/// let name (\ args expression)
/// ```
///
/// Rather, with `fn` you can just do this:
///
/// ```lisp
/// fn name args expression
/// ```
///
/// # Examples
///
/// ```lisp
/// fn double n (* 2 n)
/// double 5             ; => 10
///
/// fn add (a b) (+ a b)
/// add 10 20            ; => 30
/// ```
fn eval_fn(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 3, 3);

    let (head, tail) = args.split_first().unwrap();

    let name = match head {
        CrispExpr::Symbol(s) => s.clone(),
        _ => return type_error!("Symbol")
    };

    let lambda = eval_keyword_lambda(tail)?;
    env.data.insert(name, lambda.clone());

    Ok(lambda.clone())
}

/// `exit` exits the program with the return code given to it. If no
/// argument is given, exits with 0.
///
/// # Examples
///
/// ```lisp
/// exit
/// exit 1
/// ```
fn eval_exit(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 0, 1);

    let code = match args.first() {
        Some(CrispExpr::Number(n)) => n,
        _ => &0.0
    };

    process::exit(code.round() as i32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{env::initialize_environment, eval::eval, expr::CrispExpr::*};

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

    // let keyword

    #[test]
    fn test_let_from_eval() {
        // Tests that the let keyword calls this routine. See the rest of the tests
        // in this section for more details.
        let mut env = initialize_environment();
        let list = list![sym!("let"), sym!("foo"), Number(5.0)];
        eval(&list, &mut env).unwrap();
    }

    #[test]
    fn test_let_sets_data() {
        let mut env = initialize_environment();
        let list = vec![
            sym!("foo"),
            Number(5.0)
        ];
        eval_let(&list, &mut env).unwrap();

        assert_eq!(env.data.get("foo").unwrap(), &Number(5.0));

        // Change it

        let list = vec![
            sym!("foo"),
            Number(10.0)
        ];
        eval_let(&list, &mut env).unwrap();

        assert_eq!(env.data.get("foo").unwrap(), &Number(10.0));
    }

    #[test]
    fn test_let_evaluates() {
        let mut env = initialize_environment();
        let list = vec![
            sym!("foo"),
            list![
                sym!("+"),
                Number(1.0),
                Number(2.0)
            ]
        ];
        eval_let(&list, &mut env).unwrap();

        assert_eq!(env.data.get("foo").unwrap(), &Number(3.0));
    }

    #[test]
    fn test_let_data_retrievable() {
        // Can we get the value of the variable by `eval`ing the symbol?

        let mut env = initialize_environment();
        let list = vec![
            sym!("foo"),
            Number(5.0)
        ];
        eval_let(&list, &mut env).unwrap();

        assert_eq!(eval(&sym!("foo"), &mut env).unwrap(), Number(5.0));
    }

    // Lambdas

    #[test]
    fn test_lambda_set_to_var() {
        let mut env = initialize_environment();
        let list = list![
            sym!("let"),
            sym!("double"),
            list![
                sym!("\\"),
                list![
                    sym!("a")
                ],
                list![
                    sym!("*"),
                    sym!("a"),
                    Number(2.0)
                ]
            ]
        ];
        eval(&list, &mut env).unwrap();

        let call = list![
            sym!("double"),
            Number(5.0)
        ];

        assert_eq!(eval(&call, &mut env).unwrap(), Number(10.0));
    }

    #[test]
    fn test_lambda_single_arg() {
        // Tests passing a symbol rather than a list of symbols

        let mut env = initialize_environment();
        let list = list![
            sym!("let"),
            sym!("double"),
            list![
                sym!("\\"),
                sym!("a"),
                list![
                    sym!("*"),
                    sym!("a"),
                    Number(2.0)
                ]
            ]
        ];
        eval(&list, &mut env).unwrap();

        let call = list![
            sym!("double"),
            Number(5.0)
        ];

        assert_eq!(eval(&call, &mut env).unwrap(), Number(10.0));
    }

    #[test]
    fn test_lambda_multiple_args() {
        let mut env = initialize_environment();
        let list = list![
            sym!("let"),
            sym!("add"),
            list![
                sym!("\\"),
                list![
                    sym!("a"),
                    sym!("b")
                ],
                list![
                    sym!("+"),
                    sym!("a"),
                    sym!("b")
                ]
            ]
        ];
        eval(&list, &mut env).unwrap();

        let call = list![
            sym!("add"),
            Number(5.0),
            Number(6.0)
        ];

        assert_eq!(eval(&call, &mut env).unwrap(), Number(11.0));
    }

    #[test]
    fn test_lambda_list_head() {
        let mut env = initialize_environment();
        let call = list![
            list![
                sym!("\\"),
                list![
                    sym!("a"),
                    sym!("b")
                ],
                list![
                    sym!("+"),
                    sym!("a"),
                    sym!("b")
                ]
            ],
            Number(4.0),
            Number(2.0)
        ];

        assert_eq!(eval(&call, &mut env).unwrap(), Number(6.0));
    }

    #[test]
    fn test_lambda_nested_eval() {
        let mut env = initialize_environment();
        let call = list![
            list![
                sym!("\\"),
                list![
                    sym!("a"),
                    sym!("b")
                ],
                list![
                    sym!("+"),
                    Number(0.0),
                    list![
                        sym!("+"),
                        sym!("a"),
                        sym!("b")
                    ]
                ]
            ],
            Number(4.0),
            Number(2.0)
        ];

        assert_eq!(eval(&call, &mut env).unwrap(), Number(6.0));
    }

    #[test]
    fn test_lambda_list_err() {
        // Number as single arg (occurs on instantiation)
        let mut env = initialize_environment();
        let call = list![
            sym!("\\"),
            Number(3.0),
            list![
                sym!("+"),
                sym!("a"),
                sym!("b")
            ]
        ];

        assert!(eval(&call, &mut env).is_err());

        // Symbol in args list (occurs at lambda call)
        let mut env = initialize_environment();
        let call = list![
            list![
                sym!("\\"),
                list![
                    Number(3.0),
                    sym!("b")
                ],
                list![
                    sym!("+"),
                    sym!("a"),
                    sym!("b")
                ]
            ],
            Number(4.0),
            Number(2.0)
        ];

        assert!(eval(&call, &mut env).is_err());

        // Too few args
        let mut env = initialize_environment();
        let call = list![
            sym!("\\"),
            Number(3.0)
        ];

        assert!(eval(&call, &mut env).is_err());

        // Too many args
        let mut env = initialize_environment();
        let call = list![
            sym!("\\"),
            Number(3.0),
            list![
                sym!("+"),
                sym!("a"),
                sym!("b")
            ],
            Bool(true)
        ];

        assert!(eval(&call, &mut env).is_err());
    }

    // fn keyword

    #[test]
    fn test_fn_single_arg() {
        let mut env = initialize_environment();
        let list = list![
            sym!("fn"),
            sym!("double"),
            sym!("a"),
            list![
                sym!("*"),
                sym!("a"),
                Number(2.0)
            ]
        ];
        eval(&list, &mut env).unwrap();

        let call = list![
            sym!("double"),
            Number(5.0)
        ];

        assert_eq!(eval(&call, &mut env).unwrap(), Number(10.0));
    }

    #[test]
    fn test_fn_multiple_args() {
        let mut env = initialize_environment();
        let list = list![
            sym!("fn"),
            sym!("add"),
            list![
                sym!("a"),
                sym!("b")
            ],
            list![
                sym!("+"),
                sym!("a"),
                sym!("b")
            ]
        ];
        eval(&list, &mut env).unwrap();

        let call = list![
            sym!("add"),
            Number(5.0),
            Number(4.0)
        ];

        assert_eq!(eval(&call, &mut env).unwrap(), Number(9.0));
    }
}
