use std::fmt;

use colored::*;

#[derive(PartialEq)]
pub enum CrispError {
    ArgumentError(i32, i32),
    LoadError(String),
    ParseError(String),
    StandardError(String),
    TypeError(String)
}

macro_rules! format_error {
    ($error_type:ident, $fmt:expr, $msg:expr) => {
        format!(concat!("[{}] ", $fmt), stringify!($error_type).bright_red(), $msg).bold()
    };

    ($error_type:ident, $fmt:expr, $msg:expr, $($arg:expr),*) => {
        format!(concat!("[{}] ", $fmt), stringify!($error_type).bright_red(), $msg, $($arg),*).bold()
    };
}

impl fmt::Display for CrispError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_message = match self {
            CrispError::ArgumentError(min, max) => {
                if min == max {
                    format_error!(ArgumentError, "{} arguments expected.", min)
                } else if max < &0 {
                    format_error!(ArgumentError, "{}+ arguments expected.", min)
                } else if min < &0 {
                    format_error!(ArgumentError, "Up to {} arguments expected.", max)
                } else {
                    format_error!(ArgumentError, "{} to {} arguments expected.", min, max)
                }
            },

            CrispError::LoadError(name) => format_error!(LoadError, "No such file or directory: {}", name),
            CrispError::ParseError(msg) => format_error!(ParseError, "{}", msg),
            CrispError::StandardError(msg) => format_error!(StandardError, "{}", msg),
            CrispError::TypeError(expected) => format_error!(TypeError, "Expected {}.", expected)
        };

        write!(f, "{}", error_message)
    }
}

impl fmt::Debug for CrispError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

macro_rules! argument_error {
    (-1, $max:expr) => {
        Err(CrispError::ArgumentError(-1, $max))
    };

    ($min:expr, -1) => {
        Err(CrispError::ArgumentError($min, -1))
    };

    ($min:expr, $max:expr) => {
        Err(CrispError::ArgumentError($min, $max))
    };
}

macro_rules! check_argument_error {
    ($args:expr, -1, $max:expr) => {
        if $args.len() > $max {
            return Err(CrispError::ArgumentError(-1, $max));
        }
    };

    ($args:expr, $min:expr, -1) => {
        if $args.len() < $min {
            return Err(CrispError::ArgumentError($min, -1));
        }
    };

    ($args:expr, $min:expr, $max:expr) => {
        if $args.len() < $min || $args.len() > $max {
            return Err(CrispError::ArgumentError($min, $max));
        }
    };
}

macro_rules! generate_error_macro {
    ($macro_name:ident, $error_variant:ident) => {
        macro_rules! $macro_name {
            ($msg:expr) => {
                Err(CrispError::$error_variant($msg.to_string()))
            }
        }
    }
}

macro_rules! generate_unwrapped_error_macro {
    ($macro_name:ident, $error_variant:ident) => {
        macro_rules! $macro_name {
            ($msg:expr) => {
                CrispError::$error_variant($msg.to_string())
            }
        }
    }
}

generate_error_macro!(load_error, LoadError);
generate_error_macro!(parse_error, ParseError);
generate_unwrapped_error_macro!(parse_error_unwrapped, ParseError);
generate_error_macro!(standard_error, StandardError);
generate_error_macro!(type_error, TypeError);
