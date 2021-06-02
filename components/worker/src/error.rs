use snafu::{ResultExt, Snafu};

pub type Result<T, E = WorkerError> = std::result::Result<T, E>;

#[derive(Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum WorkerError {
    #[sanfu("WorkerManager storage error.")]
    WorkerStorageError,

    #[sanfu("App name or id {} is not registered, please register the app first.", app_name)]
    WorkerNotRegistered {
        app_name_or_id: String,
    },
    #[sanfu("server {} lookup failed", server_ip)]
    LookupFail {
        server_ip: String,
    },
}


mod outer {
    pub mod inner {
        use snafu::Snafu;

        #[derive(Debug, Snafu)]
        #[snafu(visibility = "pub(crate)")]
        pub enum Error {
            PubCrate {
                id: i32,
            },
            #[snafu(visibility = "pub(in crate::outer)")]
            PubInPath {
                id: i32,
            },
            #[snafu(visibility)]
            Private {
                id: i32,
            },
        }
    }

    #[test]
    fn can_set_default_visibility() {
        let _ = self::inner::PubCrate { id: 42 }.build();
    }

    #[test]
    fn can_set_visibility() {
        let _ = self::inner::PubInPath { id: 42 }.build();
    }
}

#[test]
fn can_set_default_visibility() {
    let _ = self::outer::inner::PubCrate { id: 42 }.build();
}