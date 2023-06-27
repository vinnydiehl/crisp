use crate::{CrispExpr, env::initialize_environment};

use std::{io::{ self, BufRead, Write }, collections::hash_map::Entry};

use crate::{print_return, send};

/// The Read/Execute/Print Loop (REPL). Continually prompts the user for
/// expressions, which it evaluates immediately and prints the return value,
/// maintaining an environment so the user may execute a program line-by-line.
pub fn run() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let env = &mut initialize_environment();

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
            Ok(ret) => print_return(&ret),
            Err(e) => eprintln!("{}", e)
        }
    }
}
