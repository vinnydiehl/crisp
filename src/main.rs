mod env;
mod error;
mod eval;
mod expr;
mod functions;
mod reader;
mod repl;

fn main() {
    repl::run();
}
