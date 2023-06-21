mod env;
mod error;
mod eval;
mod expr;
mod reader;
mod repl;

fn main() {
    repl::run();
}
