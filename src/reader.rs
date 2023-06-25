use crate::{error::{CrispError, parse_error, parse_error_unwrapped}, expr::CrispExpr};

/// Tokenizes a piece of code. `(` and `)` are their own tokens; everything
/// else is delimited by whitespace.
pub fn tokenize(mut input: String) -> Vec<String> {
    // Allow outer parens to be left off
    if !input.trim().starts_with("(") {
        input = format!("({})", input);
    }

    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_string = false;

    for ch in input.chars() {
        if in_string {
            match ch {
                '"' if current_token.chars().last().unwrap() != '\\' => {
                    current_token.push(ch);
                    tokens.push(current_token.clone());
                    current_token.clear();
                    in_string = false;
                },

                // Otherwise, just a normal character
                _ => current_token.push(ch)
            }
        } else {
            match ch {
                '"' => {
                    in_string = true;
                    current_token.push(ch);
                },

                ' ' | '\n' | '\t' => {
                    // End of non-string token
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                },

                '(' => {
                    // End of non-string token
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push("(".to_string());
                },

                ')' => {
                    // End of non-string token
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(")".to_string());
                },

                // Otherwise, we're still mid-token
                _ => current_token.push(ch)
            }
        }
    }

    tokens
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
        parse_error_unwrapped!("Couldn't get token.".to_string())
    )?;

    match &head[..] {
        "(" => parse_seq(tail),
        ")" => parse_error!("Unexpected `)`."),
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
            parse_error_unwrapped!("Couldn't find closing `)`.")
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
    match token.as_ref() {
        "true" => CrispExpr::Bool(true),
        "false" => CrispExpr::Bool(false),
        _ => token.parse().map(CrispExpr::Number)
                          .unwrap_or_else(|_| sym!(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expr::CrispExpr::*, macros::*};

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("(+ 3 var)".to_string()),
                   vec!["(", "+", "3", "var", ")"]);

        assert_eq!(tokenize("(* 5 2)".to_string()),
                   vec!["(", "*", "5", "2", ")"]);

        assert_eq!(tokenize("()".to_string()),
                   vec!["(", ")"]);

        assert_eq!(tokenize("(* 5\n    (+\t3 2))".to_string()),
                   vec!["(", "*", "5", "(", "+", "3", "2", ")", ")"]);
    }

    #[test]
    fn test_tokenize_strings() {
        assert_eq!(tokenize("(\"foo\")".to_string()),
                   vec!["(", "\"foo\"", ")"]);

        assert_eq!(tokenize("(test \"foo\" var)".to_string()),
                   vec!["(", "test", "\"foo\"", "var", ")"]);

        assert_eq!(tokenize("(test \"foo bar\" var)".to_string()),
                   vec!["(", "test", "\"foo bar\"", "var", ")"]);

        assert_eq!(tokenize("(\"test\" \"foo bar\" \"baz\")".to_string()),
                   vec!["(", "\"test\"", "\"foo bar\"", "\"baz\"", ")"]);

        assert_eq!(tokenize("(\"foo (bar) baz\")".to_string()),
                   vec!["(", "\"foo (bar) baz\"", ")"]);

        // `tokenize()` does not unescape the strings:

        assert_eq!(tokenize("(\"foo \\\"(bar)\\\" baz\")".to_string()),
                   vec!["(", "\"foo \\\"(bar)\\\" baz\"", ")"]);

        assert_eq!(tokenize("(\"foo\\n\\tbar\")".to_string()),
                   vec!["(", "\"foo\\n\\tbar\"", ")"]);

        assert_eq!(tokenize("(\"Pok\\u{00e9}mon\")".to_string()),
                   vec!["(", "\"Pok\\u{00e9}mon\"", ")"]);
    }

    #[test]
    fn test_tokenize_no_outer_parens() {
        assert_eq!(tokenize("+ 3 var".to_string()),
                   vec!["(", "+", "3", "var", ")"]);

        assert_eq!(tokenize("* 5 2".to_string()),
                   vec!["(", "*", "5", "2", ")"]);

        assert_eq!(tokenize("* 5\n    (+ 3 2)".to_string()),
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

        assert_eq!(expr, list![
             sym!("+"),
             Number(3.0),
             sym!("var")
        ]);

        assert!(remaining_tokens.is_empty());
    }

    #[test]
    fn test_parse_multi() {
        let tokens = vec!["(", "+", "5", "(", "*", "3", "2", ")", "2", ")"].into_iter()
                                                                      .map(String::from)
                                                                      .collect::<Vec<String>>();
        let (expr, remaining_tokens) = parse(&tokens).unwrap();

        assert_eq!(expr, list![
            sym!("+"),
            Number(5.0),
            list![
                sym!("*"),
                Number(3.0),
                Number(2.0)
            ],
            Number(2.0)
        ]);

        assert!(remaining_tokens.is_empty());
    }
}
