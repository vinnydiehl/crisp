#[macro_use]
#[allow(unused_imports, unused_macros)]
mod macros;

mod env;
mod error;
mod eval;
mod expr;
mod functions;
mod keywords;
mod reader;
mod repl;

fn main() {
    repl::run();
}
