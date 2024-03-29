use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;

use compact_str::CompactString;
use thiserror::Error;

mod executor;
pub use executor::Executor;

mod request;
pub use request::{Request, RequestError};

mod runtime;
pub use runtime::Runtime;

pub trait Read {
    type Request;
    fn recv(
        &mut self,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Self::Request,
                        Box<dyn StdError + Send + Sync + 'static>,
                    >,
                > + Send
                + '_,
        >,
    >;
}

#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("already exists")]
    Duplicated,
    #[error("closed")]
    Closed,
    #[error(transparent)]
    Other(#[from] Box<dyn StdError + Send + Sync + 'static>),
}

#[derive(Debug, Error)]
pub enum PairError {
    #[error("pair mismatch (expected={}, found={})", .expected, .found)]
    Mismatch {
        expected: CompactString,
        found: CompactString,
    },
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error(transparent)]
    PairError(#[from] PairError),
    #[error(transparent)]
    ConnectorError(#[from] ConnectorError),
}
