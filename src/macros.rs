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

macro_rules! num_list {
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

pub(crate) use sym;
pub(crate) use list;
pub(crate) use num_list;
pub(crate) use crisp_assert;
pub(crate) use crisp_assert_false;
pub(crate) use crisp_assert_eq;
