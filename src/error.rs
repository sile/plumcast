use fibers_rpc;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt, TrackableError};

/// This crate specific `Error` type.
#[derive(Debug, Clone)]
pub struct Error(TrackableError<ErrorKind>);
derive_traits_for_trackable_error_newtype!(Error, ErrorKind);
impl From<fibers_rpc::Error> for Error {
    fn from(f: fibers_rpc::Error) -> Self {
        // TODO
        ErrorKind::Other.takes_over(f).into()
    }
}

/// Possible error kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Input is invalid.
    InvalidInput,

    /// Other errors.
    Other,
}
impl TrackableErrorKind for ErrorKind {}
