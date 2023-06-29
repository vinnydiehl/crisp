use crate::{error::CrispError, expr::CrispExpr};

use snailquote::unescape;

/// The tokenizer alternates between these states as it scans across the input
/// character-by-character. `Scanning` is the default state, indicating that we
/// are reading tokens that are delimited by whitespace (or parens). Tokens with
/// special indicators (e.g. [`String`](CrispExpr)s) will change the tokenizer's
/// state temporarily so that it can cut and process that token according to
/// the desired rules.
enum TokenState {
    Scanning,

    Char,
    String
}

/// Tokenizes a piece of code. `(` and `)` are their own tokens; everything
/// else is delimited by whitespace.
pub fn tokenize(mut input: String) -> Vec<String> {
    // Allow outer parens to be left off
    if !input.trim().starts_with("(") {
        input = format!("({})", input);
    }

    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut state = TokenState::Scanning;

    for ch in input.chars() {
        match state {
            TokenState::Scanning => {
                match ch {
                    ',' => {
                        state = TokenState::Char;
                        current_token.push(ch);
                    }

                    '"' | '\'' => {
                        state = TokenState::String;
                        current_token.push(ch);
                    },

                    ' ' | '\n' | '\t' => {
                        // End of token
                        if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                    },

                    '(' => {
                        // End of token
                        if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                        tokens.push("(".to_string());
                    },

                    ')' => {
                        // End of token
                        if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                        tokens.push(")".to_string());
                    },

                    // Otherwise, we're still mid-token
                    _ => current_token.push(ch)
                }
            },

            TokenState::Char => {
                current_token.push(ch);
                tokens.push(current_token.clone());
                current_token.clear();
                state = TokenState::Scanning;
            },

            TokenState::String => {
                match ch {
                    '"' | '\'' if current_token.chars().last().unwrap() != '\\' => {
                        current_token.push(ch);
                        tokens.push(current_token.clone());
                        current_token.clear();
                        state = TokenState::Scanning;
                    },

                    // Otherwise, just a normal character
                    _ => current_token.push(ch)
                }
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
        _   => Ok((parse_atom(head)?, tail))
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
fn parse_atom(token: &str) -> Result<CrispExpr, CrispError> {
    let expr = match token.as_ref() {
        "true" => CrispExpr::Bool(true),
        "false" => CrispExpr::Bool(false),
        _ => {
            match token.chars().next().unwrap() {
                ',' => {
                    CrispExpr::Char(token.chars().nth(1).unwrap())
                },

                '"' | '\'' => {
                    unescape(token).map(CrispExpr::CrispString)
                                   .map_err(|_| parse_error_unwrapped!("Invalid string."))?
                },

                _ => {
                    token.parse().map(CrispExpr::Number)
                                 .unwrap_or_else(|_| sym!(token))
                }
            }
        }
    };

    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::CrispExpr::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("(+ 3 var)".to_string()),
                   vec!["(", "+", "3", "var", ")"]);

        assert_eq!(tokenize("   (* 5 2)".to_string()),
                   vec!["(", "*", "5", "2", ")"]);

        assert_eq!(tokenize("()".to_string()),
                   vec!["(", ")"]);

        assert_eq!(tokenize("(* 5\n    (+\t3 2))".to_string()),
                   vec!["(", "*", "5", "(", "+", "3", "2", ")", ")"]);
    }

    #[test]
    fn test_tokenize_chars() {
        assert_eq!(tokenize("(,a)".to_string()),
                   vec!["(", ",a", ")"]);

        assert_eq!(tokenize("(,a ,b ,c)".to_string()),
                   vec!["(", ",a", ",b", ",c", ")"]);

        assert_eq!(tokenize("(,a,b,c)".to_string()),
                   vec!["(", ",a", ",b", ",c", ")"]);
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

        assert_eq!(tokenize("('foo' '(bar) baz')".to_string()),
                   vec!["(", "'foo'", "'(bar) baz'", ")"]);

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
    fn test_parse_symbol() {
        assert_eq!(parse_atom("foo").unwrap(), sym!("foo"));
        assert_eq!(parse_atom("var-name").unwrap(), sym!("var-name"));
        assert_eq!(parse_atom("+").unwrap(), sym!("+"));
    }

    #[test]
    fn test_parse_char() {
        assert_eq!(parse_atom(",a").unwrap(), Char('a'));
        assert_eq!(parse_atom(", ").unwrap(), Char(' '));
        assert_eq!(parse_atom(",\\").unwrap(), Char('\\'));
        assert_eq!(parse_atom(",\"").unwrap(), Char('"'));
        assert_eq!(parse_atom(",'").unwrap(), Char('\''));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_atom("\"foo\"").unwrap(),
                   str!("foo"));

        assert_eq!(parse_atom("\"foo bar\"").unwrap(),
                   str!("foo bar"));

        assert_eq!(parse_atom("\"foo\n\t\rbar\"").unwrap(),
                   str!("foo\n\t\rbar"));

        assert_eq!(parse_atom("\"Pok\\u{00e9}mon\"").unwrap(),
                   str!("Pok\u{00e9}mon"));

        assert_eq!(parse_atom("'foo\n\t\r  bar'").unwrap(),
                   str!("foo\n\t\r  bar"));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_atom("0").unwrap(), Number(0.0));
        assert_eq!(parse_atom("1").unwrap(), Number(1.0));
        assert_eq!(parse_atom("3.14").unwrap(), Number(3.14));
        assert_eq!(parse_atom("420").unwrap(), Number(420.0));
        assert_eq!(parse_atom("-420").unwrap(), Number(-420.0));
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
        let tokens = vec!["(", "+", "5", "(", "*", "3", "2", ")", "2", ")"]
            .into_iter()
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
