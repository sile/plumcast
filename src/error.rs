use fibers_rpc;
use std;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt, TrackableError};

/// This crate specific `Error` type.
#[derive(Debug, Clone)]
pub struct Error(TrackableError<ErrorKind>);
derive_traits_for_trackable_error_newtype!(Error, ErrorKind);
impl From<std::sync::mpsc::RecvError> for Error {
    fn from(f: std::sync::mpsc::RecvError) -> Self {
        ErrorKind::Other.cause(f).into()
    }
}
impl From<fibers_rpc::Error> for Error {
    fn from(f: fibers_rpc::Error) -> Self {
        let kind = match f.kind() {
            fibers_rpc::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
            fibers_rpc::ErrorKind::Timeout
            | fibers_rpc::ErrorKind::Unavailable
            | fibers_rpc::ErrorKind::Other => ErrorKind::Other,
        };
        let rpc_error_kind = *f.kind();
        track!(kind.takes_over(f); rpc_error_kind).into()
    }
}

/// Possible error kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Input is invalid.
    InvalidInput,

    /// Inconsistent state.
    ///
    /// There are probably bugs in the program.
    InconsistentState,

    /// Other errors.
    Other,
}
impl TrackableErrorKind for ErrorKind {}
