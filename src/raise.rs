use std::fmt::{self, Display, Formatter};
use std::panic::{self, PanicHookInfo, UnwindSafe};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raised(String);

impl Raised {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

impl Display for Raised {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Raised {}

pub fn raise(message: impl Display) -> ! {
    panic::panic_any(Raised::new(message.to_string()));
}

pub trait RaiseExt<T, E> {
    fn raise(self) -> T;
}

impl<T, E: Display> RaiseExt<T, E> for Result<T, E> {
    fn raise(self) -> T {
        match self {
            Ok(value) => value,
            Err(error) => raise(error),
        }
    }
}

pub trait RaiseContext<T> {
    fn raise_with_context(self, message: impl FnOnce() -> String) -> T;
}

impl<T, E: Display> RaiseContext<T> for Result<T, E> {
    fn raise_with_context(self, message: impl FnOnce() -> String) -> T {
        self.map_err(|error| format!("{}: {error}", message()))
            .raise()
    }
}

impl<T> RaiseContext<T> for Option<T> {
    fn raise_with_context(self, message: impl FnOnce() -> String) -> T {
        self.ok_or_else(message).raise()
    }
}

pub fn catch_raised<T, F>(operation: F) -> Result<T, Raised>
where
    F: FnOnce() -> T + UnwindSafe,
{
    match panic::catch_unwind(operation) {
        Ok(value) => Ok(value),
        Err(payload) => match payload.downcast::<Raised>() {
            Ok(error) => Err(*error),
            Err(payload) => panic::resume_unwind(payload),
        },
    }
}

pub fn suppress_raised_hook() {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if !is_raised_error(panic_info) {
            previous_hook(panic_info);
        }
    }));
}

fn is_raised_error(panic_info: &PanicHookInfo<'_>) -> bool {
    panic_info.payload().downcast_ref::<Raised>().is_some()
}

#[macro_export]
macro_rules! catch_raised {
    ($operation:expr $(,)?) => {{ $crate::raise::catch_raised(|| $operation) }};
}

#[cfg(test)]
mod tests {
    use super::{RaiseContext, RaiseExt, Raised, catch_raised, raise};

    #[test]
    fn raise_returns_successful_values() {
        assert_eq!(Ok::<_, &str>(42).raise(), 42);
    }

    #[test]
    fn result_raise_context_adds_the_context() {
        let result = catch_raised(|| {
            Err::<(), _>("failure").raise_with_context(|| "reading file".into());
        });

        assert_eq!(result, Err(Raised::new("reading file: failure")));
    }

    #[test]
    fn option_raise_context_raises_when_missing() {
        let result = catch_raised(|| {
            None::<()>.raise_with_context(|| "missing value".into());
        });

        assert_eq!(result, Err(Raised::new("missing value")));
    }

    #[test]
    fn raise_creates_a_catchable_user_error() {
        let result = catch_raised(|| raise("expected error"));

        assert_eq!(result, Err(Raised::new("expected error")));
    }

    #[test]
    fn catch_raised_returns_raised_error() {
        let result = catch_raised(|| Err::<(), _>("expected error").raise());

        assert_eq!(result, Err(Raised::new("expected error")));
    }

    #[test]
    fn catch_raised_resumes_unrelated_panics() {
        let result = std::panic::catch_unwind(|| {
            let _ = catch_raised(|| panic!("unrelated panic"));
        });

        assert!(result.is_err());
    }

    #[test]
    fn macro_catches_raised_errors() {
        let result = crate::catch_raised!({
            Err::<(), _>("from macro").raise();
        });

        assert_eq!(result.unwrap_err().message(), "from macro");
    }
}
