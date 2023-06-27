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
use std::path::{Path, PathBuf};

use clap::{arg, command, value_parser, ArgMatches};

use env::{CrispEnv, initialize_environment};
use error::CrispError;
use eval::eval;
use expr::CrispExpr;
use reader::{parse, tokenize};

fn parse_args() -> ArgMatches {
    command!()
        .arg(arg!([input] "File to run."))
        .arg(
            arg!(-c --config <FILE> "Sets a custom config file")
                .required(false)
                .value_parser(value_parser!(PathBuf))
        )
        .arg(arg!(-d --debug ... "Display debug information"))
        .get_matches()
}

fn main() -> Result<(), CrispError> {
    let matches = parse_args();

    if let Some(filename) = matches.get_one::<String>("input") {
        if let Ok(lines) = read_lines(filename) {
            let mut env = initialize_environment();

            let mut current_expr = String::new();

            // Build onto the current expression as long as the line is indented
            for line in lines {
                if let Ok(str) = line {
                    if !current_expr.is_empty() && !str.starts_with(' ') && !str.starts_with('\t') {
                        send(current_expr.clone(), &mut env)?;
                        current_expr.clear();
                    }

                    current_expr.push_str(&str);
                    current_expr.push(' ');
                } else {
                    return standard_error!(format!("Error reading file: {}", filename));
                }
            }

            // There will be one more expression in the buffer
            send(current_expr.clone(), &mut env)?;
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

pub fn send(input: String, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let (ast, _) = parse(&tokenize(input))?;
    Ok(eval(&ast, env)?)
}
