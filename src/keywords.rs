use crate::{error::CrispError, expr::CrispExpr, env::CrispEnv, eval::eval};

pub fn eval_keyword(expr: &CrispExpr, args: &[CrispExpr],
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
}
