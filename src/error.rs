use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CrispError {
    Reason(String)
}

impl fmt::Display for CrispError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Handle each error variant and format it appropriately
            // For example:
            // CrispError::Reason(msg) => write!(f, "{}", msg),
            _ => write!(f, "An error occurred."),
        }
    }
}
