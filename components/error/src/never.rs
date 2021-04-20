use std::fmt;
use std::fmt::Formatter;

/// A type representing a value that can never materialize.
///
/// This would be `!`, but it isn't stable yet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Never {}

impl fmt::Display for Never {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl std::error::Error for Never {}
