#[macro_use]
#[allow(unused_imports, unused_macros)]
mod macros;

#[macro_use]
#[allow(unused_imports, unused_macros)]
mod error;

mod env;
mod eval;
mod expr;
mod functions;
mod keywords;
mod reader;
mod repl;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use clap::{arg, command, ArgMatches};
use colored::*;
use snailquote::escape;

use env::{CrispEnv, initialize_environment};
use error::CrispError;
use eval::eval;
use expr::CrispExpr;
use reader::{parse, tokenize};

/// Parses the CLI arguments. See the [`clap`
/// examples](https://github.com/clap-rs/clap/tree/master/examples)
/// for more information.
fn parse_args() -> ArgMatches {
    command!()
        .arg(arg!([input] "File to run."))
        .arg(arg!(-d --debug ... "Display debug information"))
        .get_matches()
}

/// Main entry point for the program. Defers to [`repl::run()`] if there is no
/// file given, otherwise runs the file.
fn main() -> Result<(), CrispError> {
    let matches = parse_args();

    let debug = matches.get_one::<u8>("debug").unwrap() > &0;

    if let Some(filename) = matches.get_one::<String>("input") {
        if let Ok(lines) = read_lines(filename) {
            let mut env = initialize_environment();

            let mut current_expr = String::new();

            // Build onto the current expression as long as the line is indented
            for line in lines {
                if let Ok(str) = line {
                    if !current_expr.is_empty() && !str.starts_with(' ') && !str.starts_with('\t') {
                        process_expr(&current_expr, &mut env, debug)?;
                        current_expr.clear();
                    }

                    if !str.is_empty() {
                        current_expr.push_str(&str);
                        current_expr.push(' ');
                    }
                } else {
                    return standard_error!(format!("Error reading file: {}", filename));
                }
            }

            // There might be one more expression in the buffer
            if !current_expr.is_empty() {
                process_expr(&current_expr, &mut env, debug)?;
            }
        } else {
            return load_error!(filename);
        }
    } else {
        repl::run();
    }

    Ok(())
}

/// Reads the lines of a file specified by the provided `filename` and returns
/// an iterator over the lines wrapped in an [`io::Result`] representing the
/// success or failure of the operation.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Local function for parsing, evaluating, and then printing the return
/// if `print_ret` is set.
fn process_expr(expr: &String, env: &mut CrispEnv, print_ret: bool) -> Result<CrispExpr, CrispError> {
    let ret = send(expr.clone(), env)?;
    if print_ret {
        print_return(&ret);
    }
    Ok(ret)
}

/// Parses and evaluates an expression from a Rust [`String`].
pub fn send(input: String, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let (ast, _) = parse(&tokenize(input))?;

    // Eval the AST
    match eval(&ast, env)? {
        // If it evals to a Func, it's a func with no arguments, we need to run it
        CrispExpr::Func(func) => eval_func(func, &[], env),
        // Otherwise we're golden
        result => Ok(result)
    }
}

/// Prints the return value from the [`CrispExpr`] `ret`, with a colored
/// indicator preceding it.
pub fn print_return(ret: &CrispExpr) {
    let ret_indicator = "=> ".bright_green();

    match ret {
        CrispExpr::CrispString(str) => println!("{}{}", ret_indicator, escape_string(str)),
        _ => println!("{}{}", ret_indicator, ret)
    }
}

/// Escapes a string literal for display e.g. in the REPL return or displaying
/// a [`List`](CrispExpr) containing [`String`](CrispExpr)s.
///
/// # Examples
///
/// ```
/// let str = escape_string("foo");
/// assert_eq!(str, "'foo'".to_string());
///
/// let str = escape_string("a'b");
/// assert_eq!(str, "\"a'b\"".to_string());
/// ```
pub fn escape_string(str: &str) -> String {
    match escape(&str) {
        escaped if &escaped == str => format!("'{}'", escaped),
        escaped => escaped.to_string()
    }
}
