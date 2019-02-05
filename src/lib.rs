//! # SNAFU
//!
//! ## Design philosophy
//!
//! SNAFU believes that it should be easy to bin one underlying error
//! type (such as [`io::Error`](std::io::Error)) into multiple
//! domain-specific errors while also optionally adding contextual
//! information.
//!
//! SNAFU is designed to be used in libraries, not just end-user applications.
//!
//! ## Quick example
//!
//! This example mimics a (very poor) authentication process that
//! opens a file, writes to a file, and checks the user's ID. While
//! two of our operations involve an [`io::Error`](std::io::Error),
//! these are different conceptual errors to us.
//!
//! SNAFU creates *context selectors* mirroring each error
//! variant. These are used with the [`context`](ResultExt::context)
//! method to provide ergonomic error handling.
//!
//! ```rust
//! use snafu::{Snafu, ResultExt, Backtrace, ErrorCompat};
//! use std::{fs, path::{Path, PathBuf}};
//!
//! #[derive(Debug, Snafu)]
//! enum Error {
//!     #[snafu_display("Could not open config from {}: {}", "filename.display()", "source")]
//!     OpenConfig { filename: PathBuf, source: std::io::Error },
//!     #[snafu_display("Could not save config to {}: {}", "filename.display()", "source")]
//!     SaveConfig { filename: PathBuf, source: std::io::Error },
//!     #[snafu_display("The user id {} is invalid", "user_id")]
//!     UserIdInvalid { user_id: i32, backtrace: Backtrace },
//! }
//!
//! type Result<T, E = Error> = std::result::Result<T, E>;
//!
//! fn log_in_user<P>(config_root: P, user_id: i32) -> Result<bool>
//! where
//!     P: AsRef<Path>,
//! {
//!     let config_root = config_root.as_ref();
//!     let filename = &config_root.join("config.toml");
//!
//!     let config = fs::read(filename).context(OpenConfig { filename })?;
//!     // Perform updates to config
//!     fs::write(filename, config).context(SaveConfig { filename })?;
//!
//!     if user_id != 42 {
//!         UserIdInvalid { user_id }.fail()?;
//!     }
//!
//!     Ok(true)
//! }
//!
//! # const CONFIG_DIRECTORY: &str = "/does/not/exist";
//! # const USER_ID: i32 = 0;
//! fn log_in() {
//!     match log_in_user(CONFIG_DIRECTORY, USER_ID) {
//!         Ok(true) => println!("Logged in!"),
//!         Ok(false) => println!("Not logged in!"),
//!         Err(e) => {
//!             eprintln!("An error occurred: {}", e);
//!             if let Some(backtrace) = ErrorCompat::backtrace(&e) {
//!                 println!("{}", backtrace);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## The `Snafu` macro
//!
//! This procedural macro implements the [`Error`](std::error::Error)
//! trait and produces the corresponding context selectors.
//!
//! ### Detailed example
//!
//! ```rust
//! use snafu::Snafu;
//! use std::path::PathBuf;
//!
//! #[derive(Debug, Snafu)]
//! enum Error {
//!     #[snafu_display("Could not open config at {}: {}", "filename.display()", "source")]
//!     OpenConfig { filename: PathBuf, source: std::io::Error },
//!     #[snafu_display("Could not open config: {}", "source")]
//!     SaveConfig { source: std::io::Error },
//!     #[snafu_display("The user id {} is invalid", "user_id")]
//!     UserIdInvalid { user_id: i32 },
//! }
//! ```
//!
//! #### Generated code
//!
//! This will generate three additional types called *context
//! selectors*:
//!
//! ```rust,ignore
//! struct OpenConfig<P> { filename: P }
//! struct SaveConfig<P> { filename: P }
//! struct UserIdInvalid<I> { user_id: I }
//! ```
//!
//! Notably:
//!
//! 1. One struct is created for each enum variant.
//! 1. The name of the struct is the same as the enum variant's name.
//! 1. The `source` and `backtrace` fields have been removed; the
//!    library will automatically handle this for you.
//! 1. Each remaining field's type has been replaced with a generic
//!    type.
//!
//! If the original variant had a `source` field, its context selector
//! will have an implementation of [`From`](std::convert::From) for a
//! `snafu::Context`:
//!
//! ```rust,ignore
//! impl<P> From<Context<Error, OpenConfig<P>>> for Error
//! where
//!     P: Into<PathBuf>,
//! ```
//!
//! Otherwise, the context selector will have an inherent method
//! `fail`:
//!
//! ```rust,ignore
//! impl<I> UserIdInvalid<I>
//! where
//!     I: Into<i32>,
//! {
//!     fn fail<T>(self) -> Result<T, Error> { /* ... */ }
//! }
//! ```
//!
//! If the original variant had a `backtrace` field, the backtrace
//! will be automatically constructed when either `From` or `fail` are
//! called.
//!
//! ### Attributes
//!
//! #### Controlling `Display`
//!
//! For backwards compatibility purposes, there are a number of ways
//! you can specify how the `Display` trait will be implemented for
//! each variant:
//!
//! - `#[snafu::display("a format string with arguments: {}", info)]`
//!
//!   No special escaping is needed; this looks just like the arguments to a call to `println!`.
//!
//! - `#[snafu_display("a format string with arguments: {}", "info")]`
//!
//!   Every argument is quoted as a string literal separately.
//!
//! - `#[snafu_display = r#"("a format string with arguments: {}", info)"#]`
//!
//!   The entire
//!
//! Each choice has the same capabilities. All of the fields of the
//! variant will be available and you can call methods on them, such
//! as `filename.display()`.
//!
//! ## Version compatibility
//!
//! SNAFU is tested and compatible back to Rust 1.18, released on
//! 2017-06-08. Compatibility is controlled by Cargo feature flags.
//!
//! ### Default
//!
//! - Targets the current stable version of Rust at the time of
//!   release of the crate. Check the Cargo.toml for the exact
//!   version.
//!
//! ### No features - supports Rust 1.18
//!
//! - Implements `Error` and `Display`.
//! - Creates context selectors.
//!
//! ### `rust_1_30` - supports Rust 1.30
//!
//! - Adds an implementation for `Error::source`
//! - Adds support for re-exporting the `Snafu` macro directly from
//!   the `snafu` crate.
//!
//! ### `unstable_display_attribute` - supports Rust Nightly
//!
//! - Adds support for the `snafu::display` attribute.
//!
//! ## Other feature flags
//!
//! ### `backtraces`
//!
//! When enabled, you can use the [`Backtrace`](Backtrace) type in
//! your enum variant. If you never use backtraces, you can omit this
//! feature to speed up compilation a small amount.

#[cfg(feature = "backtraces")]
extern crate backtrace;

#[cfg(feature = "rust_1_30")]
extern crate snafu_derive;
#[cfg(feature = "rust_1_30")]
pub use snafu_derive::Snafu;

/// A combination of an underlying error and additional information
/// about the error. It is not expected for users of this crate to
/// interact with this type.
pub struct Context<E, C> {
    /// The underlying error
    pub error: E,
    /// Information that provides a context for the underlying error
    pub context: C,
}

/// Additions to [`Result`](std::result::Result).
pub trait ResultExt<T, E>: Sized {
    /// Extend a `Result` with additional context-sensitive information.
    ///
    /// ```rust
    /// use snafu::{Snafu, ResultExt};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     Authenticating { user_name: String, user_id: i32, source: ApiError },
    /// }
    ///
    /// fn example() -> Result<(), Error> {
    ///     another_function().context(Authenticating { user_name: "admin", user_id: 42 })?;
    ///     Ok(())
    /// }
    ///
    /// # type ApiError = Box<dyn std::error::Error>;
    /// fn another_function() -> Result<i32, ApiError> {
    ///     /* ... */
    /// # Ok(42)
    /// }
    /// ```
    ///
    /// Note that the [`From`](std::convert::From) implementation
    /// generated by the macro will call
    /// [`Into::into`](std::convert::Into::into) on each field, so the
    /// types are not required to exactly match.
    fn context<C>(self, context: C) -> Result<T, Context<E, C>>;

    /// Extend a `Result` with lazily-generated context-sensitive information.
    ///
    /// ```rust
    /// use snafu::{Snafu, ResultExt};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     Authenticating { user_name: String, user_id: i32, source: ApiError },
    /// }
    ///
    /// fn example() -> Result<(), Error> {
    ///     another_function().with_context(|| Authenticating {
    ///         user_name: "admin".to_string(),
    ///         user_id: 42,
    ///     })?;
    ///     Ok(())
    /// }
    ///
    /// # type ApiError = std::io::Error;
    /// fn another_function() -> Result<i32, ApiError> {
    ///     /* ... */
    /// # Ok(42)
    /// }
    /// ```
    ///
    /// Note that this *may not* be needed in many cases because the
    /// [`From`](std::convert::From) implementation generated by the
    /// macro will call [`Into::into`](std::convert::Into::into) on
    /// each field.
    fn with_context<F, C>(self, context: F) -> Result<T, Context<E, C>>
    where
        F: FnOnce() -> C;

    /// Extend a `Result` with additional context-sensitive
    /// information and immediately convert it to another `Result`.
    ///
    /// This is most useful when using `Result`'s combinators and when
    /// the final `Result` type is already constrained.
    ///
    /// ```rust
    /// use snafu::{Snafu, ResultExt};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     Authenticating { user_name: String, user_id: i32, source: ApiError },
    /// }
    ///
    /// fn example() -> Result<i32, Error> {
    ///     another_function()
    ///         .map(|v| v + 10)
    ///         .eager_context(Authenticating { user_name: "admin", user_id: 42 })
    /// }
    ///
    /// # type ApiError = std::io::Error;
    /// fn another_function() -> Result<i32, ApiError> {
    ///     /* ... */
    /// # Ok(42)
    /// }
    /// ```
    fn eager_context<C, E2>(self, context: C) -> Result<T, E2>
    where
        E2: From<Context<E, C>>,
    {
        self.context(context).map_err(Into::into)
    }

    /// Extend a `Result` with lazily-generated context-sensitive
    /// information and immediately convert it to another `Result`.
    ///
    /// This is most useful when using `Result`'s combinators and when
    /// the final `Result` type is already constrained.
    ///
    /// ```rust
    /// use snafu::{Snafu, ResultExt};
    ///
    /// #[derive(Debug, Snafu)]
    /// enum Error {
    ///     Authenticating { user_name: String, user_id: i32, source: ApiError },
    /// }
    ///
    /// fn example() -> Result<i32, Error> {
    ///     another_function()
    ///         .map(|v| v + 10)
    ///         .with_eager_context(|| Authenticating {
    ///             user_name: "admin".to_string(),
    ///             user_id: 42,
    ///         })
    /// }
    ///
    /// # type ApiError = std::io::Error;
    /// fn another_function() -> Result<i32, ApiError> {
    ///     /* ... */
    /// # Ok(42)
    /// }
    /// ```
    ///
    /// Note that this *may not* be needed in many cases because the
    /// [`From`](std::convert::From) implementation generated by the
    /// macro will call [`Into::into`](std::convert::Into::into) on
    /// each field.
    fn with_eager_context<F, C, E2>(self, context: F) -> Result<T, E2>
    where
        F: FnOnce() -> C,
        E2: From<Context<E, C>>,
    {
        self.with_context(context).map_err(Into::into)
    }
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E> {
    fn context<C>(self, context: C) -> Result<T, Context<E, C>> {
        self.map_err(|error| Context { error, context })
    }

    fn with_context<F, C>(self, context: F) -> Result<T, Context<E, C>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|error| {
            let context = context();
            Context { error, context }
        })
    }
}

/// Backports changes to the [`Error`](std::error::Error) trait to
/// versions of Rust lacking them.
///
/// It is recommended to always call these methods explicitly so that
/// it is easy to replace usages of this trait when you start
/// supporting a newer version of Rust.
///
/// ```
/// # use snafu::{Snafu, ErrorCompat};
/// # #[derive(Debug, Snafu)] enum Example {};
/// # fn example(error: Example) {
/// ErrorCompat::backtrace(&error); // Recommended
/// error.backtrace();              // Discouraged
/// # }
/// ```
pub trait ErrorCompat {
    /// Returns a [`Backtrace`](Backtrace) that may be printed.
    #[cfg(feature = "backtraces")]
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }
}

#[cfg(feature = "backtraces")]
pub use backtrace_shim::*;

#[cfg(feature = "backtraces")]
mod backtrace_shim {
    use backtrace;
    use std::{fmt, path};

    /// A backtrace starting from the beginning of the thread.
    #[derive(Debug)]
    pub struct Backtrace(backtrace::Backtrace);

    impl Backtrace {
        /// Creates the backtrace.
        // Inlining in an attempt to remove this function from the backtrace
        #[inline(always)]
        pub fn new() -> Self {
            Backtrace(backtrace::Backtrace::new())
        }
    }

    impl Default for Backtrace {
        // Inlining in an attempt to remove this function from the backtrace
        #[inline(always)]
        fn default() -> Self {
            Self::new()
        }
    }

    impl fmt::Display for Backtrace {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let frames = self.0.frames();
            let width = (frames.len() as f32).log10().floor() as usize + 1;

            for (index, frame) in frames.iter().enumerate() {
                let mut symbols = frame.symbols().iter().map(SymbolDisplay);

                if let Some(symbol) = symbols.next() {
                    writeln!(f, "{index:width$} {name}", index = index, width = width, name = symbol.name())?;
                    if let Some(location) = symbol.location() {
                        writeln!(f, "{index:width$} {location}", index = "", width = width, location = location)?;
                    }

                    for symbol in symbols {
                        writeln!(f, "{index:width$} {name}", index = "", width = width, name = symbol.name())?;
                        if let Some(location) = symbol.location() {
                            writeln!(f, "{index:width$} {location}", index = "", width = width, location = location)?;
                        }
                    }
                }
            }

            Ok(())
        }
    }

    struct SymbolDisplay<'a>(&'a backtrace::BacktraceSymbol);

    impl<'a> SymbolDisplay<'a> {
        fn name(&self) -> SymbolNameDisplay<'a> {
            SymbolNameDisplay(self.0)
        }

        fn location(&self) -> Option<SymbolLocationDisplay<'a>> {
            self.0.filename().map(|f| SymbolLocationDisplay(self.0, f))
        }
    }

    struct SymbolNameDisplay<'a>(&'a backtrace::BacktraceSymbol);

    impl<'a> fmt::Display for SymbolNameDisplay<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0.name() {
                Some(n) => write!(f, "{}", n)?,
                None => write!(f, "<unknown>")?,
            }

            Ok(())
        }
    }

    struct SymbolLocationDisplay<'a>(&'a backtrace::BacktraceSymbol, &'a path::Path);

    impl<'a> fmt::Display for SymbolLocationDisplay<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.1.display())?;
            if let Some(l) = self.0.lineno() {
                write!(f, ":{}", l)?;
            }

            Ok(())
        }
    }
}
