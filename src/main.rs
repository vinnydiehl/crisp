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

fn main() {
    repl::run();
}
