/// Implements a source-preserving error conversion and records the conversion callsite.
///
/// This uses `Location::caller()` rather than `snafu::location!()` because the latter
/// records the lexical macro expansion site instead of the runtime conversion callsite.
#[macro_export]
macro_rules! located_from {
    ($source:ty => $error:ident::$variant:ident) => {
        impl From<$source> for $error {
            #[track_caller]
            fn from(source: $source) -> Self {
                let caller = ::std::panic::Location::caller();
                $error::$variant {
                    source,
                    location: ::snafu::Location::new(caller.file(), caller.line(), caller.column()),
                }
            }
        }
    };
}

/// Adds the standard status-code error constructor while preserving its caller location.
#[macro_export]
macro_rules! located_with_code {
    ($error:ident::$variant:ident) => {
        impl $error {
            #[track_caller]
            pub fn with_code(code: impl Into<u16>, msg: impl Into<String>) -> Self {
                let caller = ::std::panic::Location::caller();
                Self::$variant {
                    code: code.into(),
                    msg: msg.into(),
                    location: ::snafu::Location::new(caller.file(), caller.line(), caller.column()),
                }
            }
        }
    };
}
