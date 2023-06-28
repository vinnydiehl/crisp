use crate::{CrispExpr, env::initialize_environment, print_return, send};

use std::{collections::hash_map::Entry, process};

use colored::*;
use rustyline::{error::ReadlineError, DefaultEditor};

/// The Read/Execute/Print Loop (REPL). Continually prompts the user for
/// expressions, which it evaluates immediately and prints the return value,
/// maintaining an environment so the user may execute a program line-by-line.
pub fn run() {
    // Find the directory that the executable is running in; this is
    // where we will save the history file.
    let binding = std::env::current_exe()
        .unwrap()
        .canonicalize()
        .expect("The current executable should exist.");
    let dir = binding.parent()
        .expect("The current executable should be a file.")
        .to_string_lossy()
        .to_owned();
    let history_file: &str = &format!("{}/repl_history", dir);

    let mut rl = DefaultEditor::new().unwrap();
    let _ = rl.load_history(history_file);

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
            },

            Entry::Vacant(entry) => {
                entry.insert(CrispExpr::Number(0.0));
                0.0
            }
        };

        let readline = rl.readline(&format!("crisp:{:03}> ", repl_line_count));
        match readline {
            Ok(line) => {
                let str = line.as_str();

                match send(str.to_string(), env) {
                    Ok(ret) => print_return(&ret),
                    Err(e) => eprintln!("{}", e)
                };

                rl.add_history_entry(str).unwrap_or_else(|err| {
                    // Couldn't add to history, warn and continue
                    let message = match err {
                        ReadlineError::Io(io_err) => io_err.kind().to_string(),
                        _ => err.to_string()
                    };

                    eprintln!("{} {}",
                        format!("[{}] Unable to add history entry:", "Warning".yellow()).bold(),
                        message
                    );

                    true
                });
            },

            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => process::exit(130),

            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }

        rl.save_history(history_file).unwrap_or_else(|err| {
            // Couldn't save history, warn and continue
            let message = match err {
                ReadlineError::Io(io_err) => io_err.kind().to_string(),
                _ => err.to_string()
            };

            eprintln!("{} {}",
                format!("[{}] Unable to save history entry:", "Warning".yellow()).bold(),
                message
            );

            ()
        });
    }
}
