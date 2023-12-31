//! This crate allows interacting with the data stored by [`OsStr`] and
//! [`OsString`], without resorting to panics or corruption for invalid UTF-8.
//! Thus, methods can be used that are already defined on [`[u8]`][slice] and
//! [`Vec<u8>`].
//!
//! Typically, the only way to losslessly construct [`OsStr`] or [`OsString`]
//! from a byte sequence is to use `OsStr::new(str::from_utf8(bytes)?)`, which
//! requires the bytes to be valid in UTF-8. However, since this crate makes
//! conversions directly between the platform encoding and raw bytes, even some
//! strings invalid in UTF-8 can be converted.
//!
//! In most cases, [`RawOsStr`] and [`RawOsString`] should be used.
//! [`OsStrBytes`] and [`OsStringBytes`] provide lower-level APIs that are
//! easier to misuse.
//!
//! # Encoding
//!
//! The encoding of bytes returned or accepted by methods of this crate is
//! intentionally left unspecified. It may vary for different platforms, so
//! defining it would run contrary to the goal of generic string handling.
//! However, the following invariants will always be upheld:
//!
//! - The encoding will be compatible with UTF-8. In particular, splitting an
//!   encoded byte sequence by a UTF-8&ndash;encoded character always produces
//!   other valid byte sequences. They can be re-encoded without error using
//!   [`RawOsString::into_os_string`] and similar methods.
//!
//! - All characters valid in platform strings are representable. [`OsStr`] and
//!   [`OsString`] can always be losslessly reconstructed from extracted bytes.
//!
//! Note that the chosen encoding may not match how Rust stores these strings
//! internally, which is undocumented. For instance, the result of calling
//! [`OsStr::len`] will not necessarily match the number of bytes this crate
//! uses to represent the same string.
//!
//! Additionally, concatenation may yield unexpected results without a UTF-8
//! separator. If two platform strings need to be concatenated, the only safe
//! way to do so is using [`OsString::push`]. This limitation also makes it
//! undesirable to use the bytes in interchange.
//!
//! Since this encoding can change between versions and platforms, it should
//! not be used for storage. The standard library provides implementations of
//! [`OsStrExt`] and [`OsStringExt`] for various platforms, which should be
//! preferred for that use case.
//!
//! # User Input
//!
//! Traits in this crate should ideally not be used to convert byte sequences
//! that did not originate from [`OsStr`] or a related struct. The encoding
//! used by this crate is an implementation detail, so it does not make sense
//! to expose it to users.
//!
//! Crate [bstr] offers some useful alternative methods, such as
//! [`ByteSlice::to_os_str`] and [`ByteVec::into_os_string`], that are meant
//! for user input. But, they reject some byte sequences used to represent
//! valid platform strings, which would be undesirable for reliable path
//! handling. They are best used only when accepting unknown input.
//!
//! This crate is meant to help when you already have an instance of [`OsStr`]
//! and need to modify the data in a lossless way.
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file.
//!
//! ### Default Features
//!
//! - **memchr** -
//!   Changes the implementation to use crate [memchr] for better performance.
//!   This feature is useless when "raw\_os\_str" is disabled.
//!
//!   For more information, see [`RawOsStr`][memchr complexity].
//!
//! - **raw\_os\_str** -
//!   Provides:
//!   - [`iter`]
//!   - [`Pattern`]
//!   - [`OsStrBytesExt`]
//!   - [`RawOsStr`]
//!   - [`RawOsStrCow`]
//!   - [`RawOsString`]
//!
//! ### Optional Features
//!
//! - **checked\_conversions** -
//!   Provides:
//!   - [`EncodingError`]
//!   - [`OsStrBytes::from_raw_bytes`]
//!   - [`OsStringBytes::from_raw_vec`]
//!   - [`RawOsStr::cow_from_raw_bytes`]
//!   - [`RawOsString::from_raw_vec`]
//!
//!   Because this feature should not be used in libraries, the
//!   "OS_STR_BYTES_CHECKED_CONVERSIONS" environment variable must be defined
//!   during compilation.
//!
//! - **conversions** -
//!   Provides methods that require encoding conversion and may be expensive:
//!   - [`OsStrBytesExt::ends_with_os`]
//!   - [`OsStrBytesExt::starts_with_os`]
//!   - [`RawOsStr::assert_cow_from_raw_bytes`]
//!   - [`RawOsStr::ends_with_os`]
//!   - [`RawOsStr::starts_with_os`]
//!   - [`RawOsStr::to_raw_bytes`]
//!   - [`RawOsString::assert_from_raw_vec`]
//!   - [`RawOsString::into_raw_vec`]
//!   - [`OsStrBytes`]
//!   - [`OsStringBytes`]
//!
//! - **print\_bytes** -
//!   Provides implementations of [`print_bytes::ToBytes`] for [`RawOsStr`] and
//!   [`RawOsString`].
//!
//! - **uniquote** -
//!   Provides implementations of [`uniquote::Quote`] for [`RawOsStr`] and
//!   [`RawOsString`].
//!
//! ### Nightly Features
//!
//! These features are unstable, since they rely on unstable Rust features.
//!
//! - **nightly** -
//!   Changes the implementation to use the ["os\_str\_bytes" nightly
//!   feature][feature] and provides:
//!   - [`RawOsStr::as_encoded_bytes`]
//!   - [`RawOsStr::as_os_str`]
//!   - [`RawOsStr::from_encoded_bytes_unchecked`]
//!   - [`RawOsStr::from_os_str`]
//!   - [`RawOsString::from_encoded_vec_unchecked`]
//!   - [`RawOsString::into_encoded_vec`]
//!   - additional trait implementations for [`RawOsStr`] and [`RawOsString`]
//!
//!   When applicable, a "Nightly Notes" section will be added to documentation
//!   descriptions, indicating differences when this feature is enabled.
//!   However, it will not cause any breaking changes.
//!
//!   This feature will cause memory leaks for some newly deprecated methods.
//!   Therefore, it is not recommended to use this feature until the next major
//!   version, when those methods will be removed. However, it can be used to
//!   prepare for upgrading and determine impact of the new feature.
//!
//!   Because this feature should not be used in libraries, the
//!   "OS_STR_BYTES_NIGHTLY" environment variable must be defined during
//!   compilation.
//!
//! # Implementation
//!
//! Some methods return [`Cow`] to account for platform differences. However,
//! no guarantee is made that the same variant of that enum will always be
//! returned for the same platform. Whichever can be constructed most
//! efficiently will be returned.
//!
//! All traits are [sealed], meaning that they can only be implemented by this
//! crate. Otherwise, backward compatibility would be more difficult to
//! maintain for new features.
//!
//! # Complexity
//!
//! Conversion method complexities will vary based on what functionality is
//! available for the platform. At worst, they will all be linear, but some can
//! take constant time. For example, [`RawOsString::into_os_string`] might be
//! able to reuse its allocation.
//!
//! # Examples
//!
//! ```
//! # use std::io;
//! #
//! # #[cfg(feature = "raw_os_str")]
//! # {
//! # #[cfg(any())]
//! use std::env;
//! use std::fs;
//!
//! use os_str_bytes::RawOsStr;
//!
//! # mod env {
//! #   use std::env;
//! #   use std::ffi::OsString;
//! #
//! #   pub fn args_os() -> impl Iterator<Item = OsString> {
//! #       let mut file = env::temp_dir();
//! #       file.push("os_str_bytes\u{E9}.txt");
//! #       return vec![OsString::new(), file.into_os_string()].into_iter();
//! #   }
//! # }
//! #
//! for file in env::args_os().skip(1) {
//!     if !RawOsStr::new(&file).starts_with('-') {
//!         let string = "Hello, world!";
//!         fs::write(&file, string)?;
//!         assert_eq!(string, fs::read_to_string(file)?);
//!     }
//! }
//! # }
//! #
//! # Ok::<_, io::Error>(())
//! ```
//!
//! [bstr]: https://crates.io/crates/bstr
//! [`ByteSlice::to_os_str`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteSlice.html#method.to_os_str
//! [`ByteVec::into_os_string`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteVec.html#method.into_os_string
//! [feature]: https://doc.rust-lang.org/unstable-book/library-features/os-str-bytes.html
//! [memchr complexity]: RawOsStr#complexity
//! [memchr]: https://crates.io/crates/memchr
//! [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
//! [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
//! [print\_bytes]: https://crates.io/crates/print_bytes
//! [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed

#![cfg_attr(not(feature = "checked_conversions"), allow(deprecated))]
// Only require a nightly compiler when building documentation for docs.rs.
// This is a private option that should not be used.
// https://github.com/rust-lang/docs.rs/issues/147#issuecomment-389544407
// https://github.com/dylni/os_str_bytes/issues/2
#![cfg_attr(os_str_bytes_docs_rs, feature(doc_cfg))]
// Nightly is also currently required for the SGX platform.
#![cfg_attr(
    all(target_vendor = "fortanix", target_env = "sgx"),
    feature(sgx_platform)
)]
#![warn(unused_results)]

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;
use std::path::PathBuf;
use std::result;

macro_rules! if_checked_conversions {
    ( $($item:item)+ ) => {
        $(
            #[cfg(feature = "checked_conversions")]
            $item
        )+
    };
}

#[cfg(not(os_str_bytes_docs_rs))]
if_checked_conversions! {
    const _: &str = env!(
        "OS_STR_BYTES_CHECKED_CONVERSIONS",
        "The 'OS_STR_BYTES_CHECKED_CONVERSIONS' environment variable must be \
         defined to use the 'checked_conversions' feature.",
    );
}

macro_rules! if_nightly {
    ( $($item:item)+ ) => {
        $(
            #[cfg(feature = "nightly")]
            $item
        )+
    };
}

#[cfg(not(os_str_bytes_docs_rs))]
if_nightly! {
    const _: &str = env!(
        "OS_STR_BYTES_NIGHTLY",
        "The 'OS_STR_BYTES_NIGHTLY' environment variable must be defined to \
         use the 'nightly' feature.",
    );
}

#[rustfmt::skip]
macro_rules! deprecated_checked_conversion {
    ( $message:expr , $item:item ) => {
        #[cfg_attr(
            not(feature = "checked_conversions"),
            deprecated = $message
        )]
        $item
    };
}

#[rustfmt::skip]
macro_rules! deprecated_conversions {
    ( $($item:item)+ ) => {
        $(
            #[cfg_attr(
                not(feature = "conversions"),
                deprecated = "enable the 'conversions' feature"
            )]
            $item
        )+
    };
}

macro_rules! if_raw_str {
    ( $($item:item)+ ) => {
        $(
            #[cfg(feature = "raw_os_str")]
            $item
        )+
    };
}

if_raw_str! {
    macro_rules! if_not_nightly {
        ( $($item:item)+ ) => {
            $(
                #[cfg(not(feature = "nightly"))]
                $item
            )+
        };
    }

    macro_rules! if_nightly_return {
        ( $nightly_value:block $($not_nightly_token:tt)* ) => {
            #[cfg(feature = "nightly")]
            return $nightly_value;
            #[cfg(not(feature = "nightly"))]
            {
                $($not_nightly_token)*
            }
        };
    }
}

if_raw_str! {
    if_nightly! {
        macro_rules! if_conversions {
            ( $($item:item)+ ) => {
                $(
                    #[cfg(feature = "conversions")]
                    $item
                )+
            };
        }
    }
}

macro_rules! expect_encoded {
    ( $result:expr ) => {
        $result.expect("invalid raw bytes")
    };
}

#[cfg_attr(
    all(target_family = "wasm", target_os = "unknown"),
    path = "wasm/mod.rs"
)]
#[cfg_attr(windows, path = "windows/mod.rs")]
#[cfg_attr(
    not(any(all(target_family = "wasm", target_os = "unknown"), windows)),
    path = "common/mod.rs"
)]
mod imp;

#[cfg(any(
    all(
        feature = "raw_os_str",
        any(
            feature = "nightly",
            all(target_family = "wasm", target_os = "unknown"),
        ),
    ),
    windows,
))]
mod util;

if_raw_str! {
    pub mod iter;

    mod pattern;
    pub use pattern::Pattern;

    mod raw_str;
    pub use raw_str::RawOsStr;
    pub use raw_str::RawOsStrCow;
    pub use raw_str::RawOsString;
}

deprecated_checked_conversion! {
    "use `OsStrBytes::assert_from_raw_bytes` or \
     `OsStringBytes::assert_from_raw_vec` instead, or enable the \
     'checked_conversions' feature",
    /// The error that occurs when a byte sequence is not representable in the
    /// platform encoding.
    ///
    /// [`Result::unwrap`] should almost always be called on results containing
    /// this error. It should be known whether or not byte sequences are
    /// properly encoded for the platform, since [the module-level
    /// documentation][encoding] discourages using encoded bytes in
    /// interchange. Results are returned primarily to make panicking behavior
    /// explicit.
    ///
    /// On Unix, this error is never returned, but [`OsStrExt`] or
    /// [`OsStringExt`] should be used instead if that needs to be guaranteed.
    ///
    /// [encoding]: self#encoding
    /// [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
    /// [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
    /// [`Result::unwrap`]: ::std::result::Result::unwrap
    #[derive(Clone, Debug, Eq, PartialEq)]
    #[cfg_attr(
        os_str_bytes_docs_rs,
        doc(cfg(feature = "checked_conversions"))
    )]
    pub struct EncodingError(imp::EncodingError);
}

impl Display for EncodingError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for EncodingError {}

type Result<T> = result::Result<T, EncodingError>;

fn from_raw_bytes<'a, S>(string: S) -> imp::Result<Cow<'a, OsStr>>
where
    S: Into<Cow<'a, [u8]>>,
{
    match string.into() {
        Cow::Borrowed(string) => imp::os_str_from_bytes(string),
        Cow::Owned(string) => imp::os_string_from_vec(string).map(Cow::Owned),
    }
}

fn cow_os_str_into_path(string: Cow<'_, OsStr>) -> Cow<'_, Path> {
    match string {
        Cow::Borrowed(string) => Cow::Borrowed(Path::new(string)),
        Cow::Owned(string) => Cow::Owned(string.into()),
    }
}

deprecated_conversions! {
    /// A platform agnostic variant of [`OsStrExt`].
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// [module]: self
    /// [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
    #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "conversions")))]
    pub trait OsStrBytes: private::Sealed + ToOwned {
        /// Converts a byte string into an equivalent platform-native string.
        ///
        /// # Panics
        ///
        /// Panics if the string is not valid for the [unspecified encoding]
        /// used by this crate.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// use std::ffi::OsStr;
        /// # use std::io;
        ///
        /// use os_str_bytes::OsStrBytes;
        ///
        /// let os_string = env::current_exe()?;
        /// let os_bytes = os_string.to_raw_bytes();
        /// assert_eq!(os_string, OsStr::assert_from_raw_bytes(os_bytes));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [unspecified encoding]: self#encoding
        #[must_use = "method should not be used for validation"]
        #[track_caller]
        fn assert_from_raw_bytes<'a, S>(string: S) -> Cow<'a, Self>
        where
            S: Into<Cow<'a, [u8]>>;

        deprecated_checked_conversion! {
            "use `assert_from_raw_bytes` instead, or enable the \
             'checked_conversions' feature",
            /// Converts a byte string into an equivalent platform-native
            /// string.
            ///
            /// [`assert_from_raw_bytes`] should almost always be used instead.
            /// For more information, see [`EncodingError`].
            ///
            /// # Errors
            ///
            /// See documentation for [`EncodingError`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::env;
            /// use std::ffi::OsStr;
            /// # use std::io;
            ///
            /// use os_str_bytes::OsStrBytes;
            ///
            /// let os_string = env::current_exe()?;
            /// let os_bytes = os_string.to_raw_bytes();
            /// assert_eq!(os_string, OsStr::from_raw_bytes(os_bytes).unwrap());
            /// #
            /// # Ok::<_, io::Error>(())
            /// ```
            ///
            /// [`assert_from_raw_bytes`]: Self::assert_from_raw_bytes
            #[cfg_attr(
                os_str_bytes_docs_rs,
                doc(cfg(feature = "checked_conversions"))
            )]
            fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
            where
                S: Into<Cow<'a, [u8]>>;
        }

        /// Converts a platform-native string into an equivalent byte string.
        ///
        /// The returned string will use an [unspecified encoding].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::ffi::OsStr;
        ///
        /// use os_str_bytes::OsStrBytes;
        ///
        /// let string = "foobar";
        /// let os_string = OsStr::new(string);
        /// assert_eq!(string.as_bytes(), &*os_string.to_raw_bytes());
        /// ```
        ///
        /// [unspecified encoding]: self#encoding
        #[must_use]
        fn to_raw_bytes(&self) -> Cow<'_, [u8]>;
    }

    #[cfg_attr(not(feature = "conversions"), allow(useless_deprecated))]
    impl OsStrBytes for OsStr {
        #[inline]
        fn assert_from_raw_bytes<'a, S>(string: S) -> Cow<'a, Self>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            expect_encoded!(from_raw_bytes(string))
        }

        #[inline]
        fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            from_raw_bytes(string).map_err(EncodingError)
        }

        #[inline]
        fn to_raw_bytes(&self) -> Cow<'_, [u8]> {
            imp::os_str_to_bytes(self)
        }
    }

    #[cfg_attr(not(feature = "conversions"), allow(useless_deprecated))]
    impl OsStrBytes for Path {
        #[inline]
        fn assert_from_raw_bytes<'a, S>(string: S) -> Cow<'a, Self>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            cow_os_str_into_path(OsStr::assert_from_raw_bytes(string))
        }

        #[inline]
        fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
        where
            S: Into<Cow<'a, [u8]>>,
        {
            OsStr::from_raw_bytes(string).map(cow_os_str_into_path)
        }

        #[inline]
        fn to_raw_bytes(&self) -> Cow<'_, [u8]> {
            self.as_os_str().to_raw_bytes()
        }
    }
}

if_raw_str! {
    if_nightly! {
        /// An extension trait providing methods from [`RawOsStr`].
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(all(feature = "nightly", feature = "raw_os_str")))
        )]
        pub trait OsStrBytesExt: OsStrBytes {
            /// Equivalent to [`str::contains`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert!(os_string.contains("oo"));
            /// assert!(!os_string.contains("of"));
            /// ```
            #[must_use]
            fn contains<P>(&self, pat: P) -> bool
            where
                P: Pattern;

            /// Equivalent to [`str::ends_with`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert!(os_string.ends_with("bar"));
            /// assert!(!os_string.ends_with("foo"));
            /// ```
            #[must_use]
            fn ends_with<P>(&self, pat: P) -> bool
            where
                P: Pattern;

            if_conversions! {
                /// Equivalent to [`str::ends_with`] but accepts this type for
                /// the pattern.
                ///
                /// # Examples
                ///
                /// ```
                /// use std::ffi::OsStr;
                ///
                /// use os_str_bytes::OsStrBytesExt;
                ///
                /// let os_string = OsStr::new("foobar");
                /// assert!(os_string.ends_with_os(OsStr::new("bar")));
                /// assert!(!os_string.ends_with_os(OsStr::new("foo")));
                /// ```
                #[cfg_attr(
                    os_str_bytes_docs_rs,
                    doc(cfg(feature = "conversions"))
                )]
                #[must_use]
                fn ends_with_os(&self, pat: &Self) -> bool;
            }

            /// Equivalent to [`str::find`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert_eq!(Some(1), os_string.find("o"));
            /// assert_eq!(None, os_string.find("of"));
            /// ```
            #[must_use]
            fn find<P>(&self, pat: P) -> Option<usize>
            where
                P: Pattern;

            /// Equivalent to [`str::rfind`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert_eq!(Some(2), os_string.rfind("o"));
            /// assert_eq!(None, os_string.rfind("of"));
            /// ```
            #[must_use]
            fn rfind<P>(&self, pat: P) -> Option<usize>
            where
                P: Pattern;

            /// Equivalent to [`str::rsplit_once`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert_eq!(
            ///     Some((OsStr::new("fo"), OsStr::new("bar"))),
            ///     os_string.rsplit_once("o"),
            /// );
            /// assert_eq!(None, os_string.rsplit_once("of"));
            /// ```
            #[must_use]
            fn rsplit_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
            where
                P: Pattern;

            /// Equivalent to [`str::split_at`].
            ///
            /// # Panics
            ///
            /// Panics if the index is not a [valid boundary].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert_eq!(
            ///     ((OsStr::new("fo"), OsStr::new("obar"))),
            ///     os_string.split_at(2),
            /// );
            /// ```
            ///
            /// [valid boundary]: RawOsStr#indices
            #[must_use]
            #[track_caller]
            fn split_at(&self, mid: usize) -> (&Self, &Self);

            /// Equivalent to [`str::split_once`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert_eq!(
            ///     Some((OsStr::new("f"), OsStr::new("obar"))),
            ///     os_string.split_once("o"),
            /// );
            /// assert_eq!(None, os_string.split_once("of"));
            /// ```
            #[must_use]
            fn split_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
            where
                P: Pattern;

            /// Equivalent to [`str::starts_with`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("foobar");
            /// assert!(os_string.starts_with("foo"));
            /// assert!(!os_string.starts_with("bar"));
            /// ```
            #[must_use]
            fn starts_with<P>(&self, pat: P) -> bool
            where
                P: Pattern;

            if_conversions! {
                /// Equivalent to [`str::starts_with`] but accepts this type
                /// for the pattern.
                ///
                /// # Examples
                ///
                /// ```
                /// use std::ffi::OsStr;
                ///
                /// use os_str_bytes::OsStrBytesExt;
                ///
                /// let os_string = OsStr::new("foobar");
                /// assert!(os_string.starts_with_os(OsStr::new("foo")));
                /// assert!(!os_string.starts_with_os(OsStr::new("bar")));
                /// ```
                #[cfg_attr(
                    os_str_bytes_docs_rs,
                    doc(cfg(feature = "conversions"))
                )]
                #[must_use]
                fn starts_with_os(&self, pat: &Self) -> bool;
            }

            /// Equivalent to [`str::strip_prefix`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("111foo1bar111");
            /// assert_eq!(
            ///     Some(OsStr::new("11foo1bar111")),
            ///     os_string.strip_prefix("1"),
            /// );
            /// assert_eq!(None, os_string.strip_prefix("o"));
            /// ```
            #[must_use]
            fn strip_prefix<P>(&self, pat: P) -> Option<&Self>
            where
                P: Pattern;

            /// Equivalent to [`str::strip_suffix`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("111foo1bar111");
            /// assert_eq!(
            ///     Some(OsStr::new("111foo1bar11")),
            ///     os_string.strip_suffix("1"),
            /// );
            /// assert_eq!(None, os_string.strip_suffix("o"));
            /// ```
            #[must_use]
            fn strip_suffix<P>(&self, pat: P) -> Option<&Self>
            where
                P: Pattern;

            /// Equivalent to [`str::trim_end_matches`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("111foo1bar111");
            /// assert_eq!("111foo1bar", os_string.trim_end_matches("1"));
            /// assert_eq!("111foo1bar111", os_string.trim_end_matches("o"));
            /// ```
            #[must_use]
            fn trim_end_matches<P>(&self, pat: P) -> &Self
            where
                P: Pattern;

            /// Equivalent to [`str::trim_matches`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("111foo1bar111");
            /// assert_eq!("foo1bar", os_string.trim_matches("1"));
            /// assert_eq!("111foo1bar111", os_string.trim_matches("o"));
            /// ```
            #[must_use]
            fn trim_matches<P>(&self, pat: P) -> &Self
            where
                P: Pattern;

            /// Equivalent to [`str::trim_start_matches`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::ffi::OsStr;
            ///
            /// use os_str_bytes::OsStrBytesExt;
            ///
            /// let os_string = OsStr::new("111foo1bar111");
            /// assert_eq!("foo1bar111", os_string.trim_start_matches("1"));
            /// assert_eq!("111foo1bar111", os_string.trim_start_matches("o"));
            /// ```
            #[must_use]
            fn trim_start_matches<P>(&self, pat: P) -> &Self
            where
                P: Pattern;
        }

        impl OsStrBytesExt for OsStr {
            #[inline]
            fn contains<P>(&self, pat: P) -> bool
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).contains(pat)
            }

            #[inline]
            fn ends_with<P>(&self, pat: P) -> bool
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).ends_with(pat)
            }

            if_conversions! {
                #[inline]
                fn ends_with_os(&self, pat: &Self) -> bool {
                    RawOsStr::from_os_str(self)
                        .ends_with_os(RawOsStr::from_os_str(pat))
                }
            }

            #[inline]
            fn find<P>(&self, pat: P) -> Option<usize>
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).find(pat)
            }

            #[inline]
            fn rfind<P>(&self, pat: P) -> Option<usize>
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).rfind(pat)
            }

            #[inline]
            fn rsplit_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self)
                    .rsplit_once(pat)
                    .map(|(prefix, suffix)| {
                        (prefix.as_os_str(), suffix.as_os_str())
                    })
            }

            #[inline]
            fn split_at(&self, mid: usize) -> (&Self, &Self) {
                let (prefix, suffix) =
                    RawOsStr::from_os_str(self).split_at(mid);
                (prefix.as_os_str(), suffix.as_os_str())
            }

            #[inline]
            fn split_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self)
                    .split_once(pat)
                    .map(|(prefix, suffix)| {
                        (prefix.as_os_str(), suffix.as_os_str())
                    })
            }

            #[inline]
            fn starts_with<P>(&self, pat: P) -> bool
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).starts_with(pat)
            }

            if_conversions! {
                #[inline]
                fn starts_with_os(&self, pat: &Self) -> bool {
                    RawOsStr::from_os_str(self)
                        .starts_with_os(RawOsStr::from_os_str(pat))
                }
            }

            #[inline]
            fn strip_prefix<P>(&self, pat: P) -> Option<&Self>
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self)
                    .strip_prefix(pat)
                    .map(RawOsStr::as_os_str)
            }

            #[inline]
            fn strip_suffix<P>(&self, pat: P) -> Option<&Self>
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self)
                    .strip_suffix(pat)
                    .map(RawOsStr::as_os_str)
            }

            #[inline]
            fn trim_end_matches<P>(&self, pat: P) -> &Self
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).trim_end_matches(pat).as_os_str()
            }

            #[inline]
            fn trim_matches<P>(&self, pat: P) -> &Self
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).trim_matches(pat).as_os_str()
            }

            #[inline]
            fn trim_start_matches<P>(&self, pat: P) -> &Self
            where
                P: Pattern,
            {
                RawOsStr::from_os_str(self).trim_start_matches(pat).as_os_str()
            }
        }
    }
}

deprecated_conversions! {
    /// A platform agnostic variant of [`OsStringExt`].
    ///
    /// For more information, see [the module-level documentation][module].
    ///
    /// [module]: self
    /// [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
    #[cfg_attr(os_str_bytes_docs_rs, doc(cfg(any(feature = "conversions"))))]
    pub trait OsStringBytes: private::Sealed + Sized {
        /// Converts a byte string into an equivalent platform-native string.
        ///
        /// # Panics
        ///
        /// Panics if the string is not valid for the [unspecified encoding]
        /// used by this crate.
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// use std::ffi::OsString;
        /// # use std::io;
        ///
        /// use os_str_bytes::OsStringBytes;
        ///
        /// let os_string = env::current_exe()?;
        /// let os_bytes = os_string.clone().into_raw_vec();
        /// assert_eq!(os_string, OsString::assert_from_raw_vec(os_bytes));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [unspecified encoding]: self#encoding
        #[must_use = "method should not be used for validation"]
        #[track_caller]
        fn assert_from_raw_vec(string: Vec<u8>) -> Self;

        deprecated_checked_conversion! {
            "use `assert_from_raw_vec` instead, or enable the \
             'checked_conversions' feature",
            /// Converts a byte string into an equivalent platform-native
            /// string.
            ///
            /// [`assert_from_raw_vec`] should almost always be used instead.
            /// For more information, see [`EncodingError`].
            ///
            /// # Errors
            ///
            /// See documentation for [`EncodingError`].
            ///
            /// # Examples
            ///
            /// ```
            /// use std::env;
            /// use std::ffi::OsString;
            /// # use std::io;
            ///
            /// use os_str_bytes::OsStringBytes;
            ///
            /// let os_string = env::current_exe()?;
            /// let os_bytes = os_string.clone().into_raw_vec();
            /// assert_eq!(
            ///     os_string,
            ///     OsString::from_raw_vec(os_bytes).unwrap(),
            /// );
            /// #
            /// # Ok::<_, io::Error>(())
            /// ```
            ///
            /// [`assert_from_raw_vec`]: Self::assert_from_raw_vec
            #[cfg_attr(
                os_str_bytes_docs_rs,
                doc(cfg(feature = "checked_conversions"))
            )]
            fn from_raw_vec(string: Vec<u8>) -> Result<Self>;
        }

        /// Converts a platform-native string into an equivalent byte string.
        ///
        /// The returned string will use an [unspecified encoding].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::ffi::OsString;
        ///
        /// use os_str_bytes::OsStringBytes;
        ///
        /// let string = "foobar".to_owned();
        /// let os_string: OsString = string.clone().into();
        /// assert_eq!(string.into_bytes(), os_string.into_raw_vec());
        /// ```
        ///
        /// [unspecified encoding]: self#encoding
        #[must_use]
        fn into_raw_vec(self) -> Vec<u8>;
    }

    #[cfg_attr(not(feature = "conversions"), allow(useless_deprecated))]
    impl OsStringBytes for OsString {
        #[inline]
        fn assert_from_raw_vec(string: Vec<u8>) -> Self {
            expect_encoded!(imp::os_string_from_vec(string))
        }

        #[inline]
        fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
            imp::os_string_from_vec(string).map_err(EncodingError)
        }

        #[inline]
        fn into_raw_vec(self) -> Vec<u8> {
            imp::os_string_into_vec(self)
        }
    }

    #[cfg_attr(not(feature = "conversions"), allow(useless_deprecated))]
    impl OsStringBytes for PathBuf {
        #[inline]
        fn assert_from_raw_vec(string: Vec<u8>) -> Self {
            OsString::assert_from_raw_vec(string).into()
        }

        #[inline]
        fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
            OsString::from_raw_vec(string).map(Into::into)
        }

        #[inline]
        fn into_raw_vec(self) -> Vec<u8> {
            self.into_os_string().into_raw_vec()
        }
    }
}

mod private {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    if_raw_str! {
        use std::borrow::Cow;

        use super::RawOsStr;
    }

    pub trait Sealed {}

    impl Sealed for char {}
    impl Sealed for OsStr {}
    impl Sealed for OsString {}
    impl Sealed for Path {}
    impl Sealed for PathBuf {}
    impl Sealed for &str {}
    impl Sealed for &String {}

    if_raw_str! {
        impl Sealed for Cow<'_, RawOsStr> {}
    }
}
