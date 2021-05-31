mod error;
mod job_fetcher;
pub mod worker_manager;

pub use error::Result;

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
