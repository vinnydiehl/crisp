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

fn parse_args() -> ArgMatches {
    command!()
        .arg(arg!([input] "File to run."))
        .arg(arg!(-d --debug ... "Display debug information"))
        .get_matches()
}

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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn process_expr(expr: &String, env: &mut CrispEnv, debug: bool) -> Result<CrispExpr, CrispError> {
    let ret = send(expr.clone(), env)?;
    if debug {
        print_return(&ret);
    }
    Ok(ret)
}

pub fn send(input: String, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let (ast, _) = parse(&tokenize(input))?;
    Ok(eval(&ast, env)?)
}

pub fn print_return(ret: &CrispExpr) {
    let ret_indicator = "=>".bright_green();

    match ret {
        CrispExpr::CrispString(str) => println!("{} {}", ret_indicator, escape_string(str)),
        _ => println!("{} {}", ret_indicator, ret)
    }
}

pub fn escape_string(str: &str) -> String {
    match escape(&str) {
        escaped if &escaped == str => format!("'{}'", escaped),
        escaped => escaped.to_string()
    }
}
