use rbatis::crud::CRUDTable;
use snafu::{ResultExt, Snafu};
use std::fmt::Display;

pub type Result<T, E = rbatis::Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum StorageError {
    #[snafu(display("Storage saved failure; msg: {}, error : {}", msg, source))]
    SaveError { source: rbatis::Error, msg: String },
    // #[snafu(display("Storage batch saved failure; msg : {}, error : {}", msg.display(), source))]
    // SaveBatchError { msg: T, source: rbatis::Error },
    // #[snafu(display("Storage deleted failure; msg: {}, error : {}", msg.display(), source))]
    // DeleteError { msg: T, source: rbatis::Error },
    // #[snafu(display("Storage deleted failure; msg: {}, error : {}", msg.display(), source))]
    // DeleteBatchError { msg: T, source: rbatis::Error },
    // #[snafu(display("Storage fetch failure; msg: {}, error : {}", msg.display(), source))]
    // FetchError { msg: T, source: rbatis::Error },
    // #[snafu(display("Storage batch fetch failure; msg: {}, error : {}", msg.display(), source))]
    // FetchBatchError { msg: T, source: rbatis::Error },
    // #[snafu(display("Storage list fetch failure; msg: {}, error : {}", msg, source))]
    // FetchListError { msg: &'a str, source: rbatis::Error },
    // #[snafu(display("Storage update failure; msg: {}, error : {}", msg.display(), source))]
    // UpdateError { msg: T, source: rbatis::Error },
}
