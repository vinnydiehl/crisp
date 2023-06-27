use crate::{error::CrispError, expr::CrispExpr, env::CrispEnv};

use dyn_fmt::AsStrFormatExt;

pub fn crisp_format(args: &[CrispExpr], _env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    if let Some((format_str, format_args)) = args.split_first() {
        return Ok(str!(match format_args {
            [] => format!("{}", format_str),
            _ => format_str.to_string().format(format_args)
        }));
    }

    argument_error!(1, -1)
}

pub fn crisp_puts(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let value = crisp_format(args, env)?;
    println!("{}", value);

    Ok(value)
}

pub fn crisp_print(args: &[CrispExpr], env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let value = crisp_format(args, env)?;
    print!("{}", value);

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expr::CrispExpr::*, env::initialize_environment};

    #[test]
    fn test_format() {
        let mut env = initialize_environment();

        let result = crisp_format(&vec![
            str!("test")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test"));

        let result = crisp_format(&vec![
            str!("test: {}"),
            str!("foo")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo"));

        let result = crisp_format(&vec![
            str!("test: {}"),
            Number(123.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: 123"));

        let result = crisp_format(&vec![
            str!("{}{}"),
            Number(1.0),
            Number(2.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("12"));

        let result = crisp_format(&vec![
            str!("{} a {} b {}"),
            str!("1"),
            Number(2.0),
            Bool(true),
        ], &mut env).unwrap();

        assert_eq!(result, str!("1 a 2 b true"));
    }

    #[test]
    fn test_format_escape() {
        let mut env = initialize_environment();

        let result = crisp_format(&vec![
            str!("{{}}"),
            Number(42.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("{}"));

        let result = crisp_format(&vec![
            str!("{}{{}}{}"),
            Number(24.0),
            Number(42.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("24{}42"));

        let result = crisp_format(&vec![
            str!("test {{ escaped braces }} {{:3}}"),
            Number(42.0)
        ], &mut env).unwrap();

        assert_eq!(result, str!("test { escaped braces } {:3}"));

        // With no arguments, braces don't need to be escaped
        let result = crisp_format(&vec![
            str!("test {{ escaped braces }} {{:3}}"),
        ], &mut env).unwrap();

        assert_eq!(result, str!("test {{ escaped braces }} {{:3}}"));
    }

    #[test]
    fn test_format_too_many_args() {
        let mut env = initialize_environment();

        // It should discard the superfluous args
        let result = crisp_format(&vec![
            str!("test: {}"),
            str!("foo"),
            str!("bar"),
            str!("baz")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo"));
    }

    #[test]
    fn test_format_too_few_args() {
        let mut env = initialize_environment();

        // It should fill in left-to-right and leave the remaining braces
        let result = crisp_format(&vec![
            str!("test: {} {} {}"),
            str!("foo")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo  "));

        let result = crisp_format(&vec![
            str!("test: {} {} {}"),
            str!("foo"),
            str!("bar")
        ], &mut env).unwrap();

        assert_eq!(result, str!("test: foo bar "));
    }
}
