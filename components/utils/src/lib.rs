pub mod component;
pub mod id_generator;
pub mod pair;
pub mod plugin;
pub mod sched_pool;
pub mod signal;
pub mod signal_handler;
pub mod time;
pub mod toml_parser;
mod yatp_pool;

/// Represents a value of one of two possible types(a more generic Result.)
#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    #[inline]
    pub fn as_ref(&self) -> Either<&L, &R> {
        match *self {
            Either::Left(ref l) => Either::Left(l),
            Either::Right(ref r) => Either::Right(r),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match *self {
            Either::Left(ref mut l) => Either::Left(l),
            Either::Right(ref mut r) => Either::Right(r),
        }
    }

    #[inline]
    pub fn left(self) -> Option<L> {
        match self {
            Either::Left(l) => Some(l),
            _ => None,
        }
    }

    #[inline]
    pub fn right(self) -> Option<R> {
        match self {
            Either::Right(r) => Some(r),
            _ => None,
        }
    }
}

use snafu::{ResultExt, Snafu};
use std::{fs, io, path::PathBuf};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Unable to read configuration from {}: {}", path.display(), source))]
    ReadConfiguration { source: io::Error, path: PathBuf },

    #[snafu(display("Unable to write result to {}: {}", path.display(), source))]
    WriteResult { source: io::Error, path: PathBuf },
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn process_data() -> Result<()> {
    let path = "config.toml";
    let configuration = fs::read_to_string(path).context(ReadConfiguration { path })?;
    let path = unpack_config(&configuration);
    fs::write(&path, b"My complex calculation").context(WriteResult { path })?;
    Ok(())
}

fn unpack_config(data: &str) -> &str {
    "/some/path/that/does/not/exist"
}
