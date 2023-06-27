use crate::{env::{CrispEnv, initialize_environment}, error::CrispError,
            eval::eval, expr::CrispExpr, reader::{parse, tokenize}};

use std::{io::{ self, BufRead, Write }, collections::hash_map::Entry};

use colored::*;
use snailquote::escape;

pub fn run() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let env = &mut initialize_environment();

    let ret_indicator = "=>".bright_green();

    loop {
        // Increment/get the current line count. If the value is
        // empty or has become corrupted, reset it to zero.
        let repl_line_count = match env.data.entry("crisp_repl_line_count".to_string()) {
            Entry::Occupied(mut entry) => {
                let value = entry.get_mut();
                match value {
                    CrispExpr::Number(n) => {
                        *n += 1.0;
                        *n
                    },
                    _ => {
                        *value = CrispExpr::Number(0.0);
                        0.0
                    }
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(CrispExpr::Number(0.0));
                0.0
            }
        };

        print!("crisp:{:03}> ", repl_line_count);
        stdout.flush().unwrap();

        let line = stdin.lock().lines().next().unwrap().unwrap();

        if line.trim() == "quit" {
            break;
        }

        match send(line, env) {
            Ok(ret) => {
                match ret {
                    CrispExpr::CrispString(str) => {
                        let s = match escape(&str) {
                            escaped if escaped == str => format!("'{}'", escaped),
                            escaped => escaped.to_string()
                        };
                        println!("{} {}", ret_indicator, s);
                    }
                    _ => println!("{} {}", ret_indicator, ret)
                }
            }
            Err(e) => eprintln!("{}", e)
        }
    }
}

fn send(input: String, env: &mut CrispEnv) -> Result<CrispExpr, CrispError> {
    let (ast, _) = parse(&tokenize(input))?;
    Ok(eval(&ast, env)?)
}
