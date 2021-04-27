pub mod signal;
pub mod drain;
pub mod id_generator;
pub mod signal_handler;

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