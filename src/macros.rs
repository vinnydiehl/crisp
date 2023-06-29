macro_rules! str {
    ($name:expr) => {
        CrispExpr::CrispString($name.to_string())
    }
}

macro_rules! sym {
    ($name:expr) => {
        CrispExpr::Symbol($name.to_string())
    }
}

macro_rules! list {
    ($($elem:expr),*) => {
        CrispExpr::List(vec![$($elem),*])
    }
}

macro_rules! lambda {
    (args: [$($arg:expr),*], func: [$($func:expr),*]) => {{
        CrispExpr::Lambda(CrispLambda {
            args: Rc::new(list![$(sym!($arg)),*]),
            func: Rc::new(list![$($func),*])
        })
    }};
}

macro_rules! bool_list {
    ($($elem:expr),*) => {
        CrispExpr::List(vec![$(CrispExpr::Bool($elem)),*])
    }
}

macro_rules! bool_vec {
    ($($elem:expr),*) => {
        vec![$(CrispExpr::Bool($elem)),*]
    }
}

macro_rules! num_list {
    ($($elem:expr),*) => {
        CrispExpr::List(vec![$(CrispExpr::Number($elem)),*])
    }
}

macro_rules! num_vec {
    ($($elem:expr),*) => {
        vec![$(CrispExpr::Number($elem)),*]
    }
}

macro_rules! crisp_assert {
    ($expr:expr) => {
        assert_eq!($expr.unwrap(), CrispExpr::Bool(true));
    }
}

macro_rules! crisp_assert_false {
    ($expr:expr) => {
        assert_eq!($expr.unwrap(), CrispExpr::Bool(false));
    }
}

macro_rules! crisp_assert_eq {
    ($expr:expr, $result:expr) => {
        assert_eq!($expr.unwrap(), CrispExpr::Number($result));
    }
}
