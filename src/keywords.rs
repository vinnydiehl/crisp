use std::rc::Rc;

use crate::{error::{CrispError, check_argument_error, type_error},
            expr::{CrispExpr, CrispLambda}, env::CrispEnv, eval::eval};

pub fn eval_keyword(expr: &CrispExpr, args: &[CrispExpr],
                env: &mut CrispEnv) -> Option<Result<CrispExpr, CrispError>> {
    match expr {
        CrispExpr::Symbol(s) => {
            match s.as_ref() {
                "if" => Some(eval_if(args, env)),
                "let" => Some(eval_let(args, env)),
                "\\" => Some(eval_lambda(args)),
                _ => None
            }
        },
        _ => None
    }
}

fn eval_if(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 3, 3);

    match eval(args.first().unwrap(), env)? {
        CrispExpr::Bool(b) => {
            // The function is going to be called like:
            //     (if (> a b) true_routine false_routine)
            // Depending on whether or not the predicate is true, we want to index
            // the args differently (0 is the predicate)
            let response = args.get(if b { 1 } else { 2 }).unwrap();

            eval(response, env)
        },
        _ => type_error!("Bool")
    }
}

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

fn eval_lambda(args: &[CrispExpr]) -> Result<CrispExpr, CrispError> {
    check_argument_error!(args, 2, 2);

    Ok(CrispExpr::Lambda(CrispLambda {
        args: Rc::new(args.first().unwrap().clone()),
        func: Rc::new(args.get(1).unwrap().clone())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{env::initialize_environment, eval::eval, expr::CrispExpr::*, macros::*};

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
}
