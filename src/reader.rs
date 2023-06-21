use crate::{error::CrispError, expr::CrispExpr};

/// Tokenizes a piece of code. `(` and `)` are their own tokens; everything
/// else is delimited by whitespace.
pub fn tokenize(str: String) -> Vec<String> {
    // Force whitespace around parens
    str.replace("(", " ( ")
       .replace(")", " ) ")
       .split_whitespace()
       .map(|x| x.to_string())
       .collect()
}

/// Parses an expression from a slice of tokens.
///
/// # Returns
///
/// * `Ok((expr, rest))` if parsing is successful, where `expr` is the parsed
///   expression and `rest` is the remaining unparsed tokens.
/// * `Err(error)` if an error occurs during parsing.
pub fn parse<'a>(tokens: &'a[String]) -> Result<(CrispExpr, &'a[String]), CrispError> {
    let (head, tail) = tokens.split_first().ok_or_else(||
        CrispError::Reason("Couldn't get token.".to_string())
    )?;

    match &head[..] {
        "(" => parse_seq(tail),
        ")" => Err(CrispError::Reason("Unexpected `)`.".to_string())),
        _   => Ok((parse_atom(head), tail))
    }
}

/// Parses a sequence after an opening `(`, all the way up until the closing `)`.
/// This calls [`parse()`] to parse the atom, and recurses back and forth with it
/// if necessary to handle nesting.
fn parse_seq<'a>(token_slice: &'a[String]) -> Result<(CrispExpr, &'a[String]), CrispError> {
    let mut res: Vec<CrispExpr> = vec![];
    let mut tokens = token_slice;

    loop {
        let (head, tail) = tokens.split_first().ok_or_else(||
            CrispError::Reason("Couldn't find closing `)`.".to_string())
        )?;

        if head == ")" {
            // Skip closing `)`
            return Ok((CrispExpr::List(res), tail))
        }

        let (expr, unparsed) = parse(&tokens)?;
        res.push(expr);
        tokens = unparsed;
    }
}

/// Parses an atom out of an individual token.
fn parse_atom(token: &str) -> CrispExpr {
    token.parse().map(CrispExpr::Number)
                 .unwrap_or_else(|_| CrispExpr::Symbol(token.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::CrispExpr::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("(+ 3 var)".to_string()),
                   vec!["(", "+", "3", "var", ")"]);

        assert_eq!(tokenize("(* 5 2)".to_string()),
                   vec!["(", "*", "5", "2", ")"]);

        assert_eq!(tokenize("()".to_string()),
                   vec!["(", ")"]);

        assert_eq!(tokenize("(* 5\n    (+ 3 2))".to_string()),
                   vec!["(", "*", "5", "(", "+", "3", "2", ")", ")"]);
    }

    #[test]
    fn test_parse_atom() {
        assert_eq!(parse_atom("+"),
                   Symbol("+".to_string()));

        assert_eq!(parse_atom("3.14"),
                   Number(3.14));
    }

    #[test]
    fn test_parse() {
        let tokens = vec!["(", "+", "3", "var", ")"].into_iter()
                                                    .map(String::from)
                                                    .collect::<Vec<String>>();

        let (expr, remaining_tokens) = parse(&tokens).unwrap();

        assert_eq!(expr, List(vec![
             Symbol("+".to_string()),
             Number(3.0),
             Symbol("var".to_string())
        ]));

        assert!(remaining_tokens.is_empty());
    }

    #[test]
    fn test_parse_multi() {
        let tokens = vec!["(", "+", "5", "(", "*", "3", "2", ")", "2", ")"].into_iter()
                                                                      .map(String::from)
                                                                      .collect::<Vec<String>>();
        let (expr, remaining_tokens) = parse(&tokens).unwrap();

        assert_eq!(expr, List(vec![
            Symbol("+".to_string()),
            Number(5.0),
            List(vec![
                Symbol("*".to_string()),
                Number(3.0),
                Number(2.0),
            ]),
            Number(2.0)
        ]));

        assert!(remaining_tokens.is_empty());
    }
}
