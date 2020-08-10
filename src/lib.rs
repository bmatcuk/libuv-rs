#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate libuv_sys2 as uv;

macro_rules! callbacks {
    ($($v:vis $Name:ident($($($a:ident: $T:ty),+)?)$( -> $TReturn:ty)?);+;) => {
        $(__callback! {
            $v $Name($($($a: $T),+)?)$(->$TReturn)?
        })+
    }
}

macro_rules! __callback {
    ($v:vis $Name:ident($($($a:ident: $T:ty),+)?)$( -> $TReturn:ty)?) => {
        $v enum $Name<'a> {
            CB(Box<dyn FnMut($($($T),+)?)$(->$TReturn)? + 'a>),
            Nil
        }

        impl<'a, T, F> From<(&'a mut T, F)> for $Name<'a>
        where
            F: Fn(&mut T$(,$($T),+)?)$(->$TReturn)? + 'static,
        {
            fn from(t: (&'a mut T, F)) -> $Name<'a> {
                $Name::CB(Box::new(move |$($($a),+)?| (t.1)(t.0$(,$($a),+)?)))
            }
        }

        impl<'a, T, F> From<(&'a T, F)> for $Name<'a>
        where
            F: Fn(&T$(,$($T),+)?)$(->$TReturn)? + 'static,
        {
            fn from(t: (&'a T, F)) -> $Name<'a> {
                $Name::CB(Box::new(move |$($($a),+)?| (t.1)(t.0$(,$($a),+)?)))
            }
        }

        impl<'a, F> From<F> for $Name<'a>
        where
            F: FnMut($($($T),+)?)$(->$TReturn)? + 'a
        {
            fn from(f: F) -> $Name<'a> {
                $Name::CB(Box::new(f))
            }
        }

        impl<'a> From<()> for $Name<'a> {
            fn from(_: ()) -> $Name<'a> {
                $Name::Nil
            }
        }

        impl<'a> $Name<'a> {
            pub fn new<CB: Into<$Name<'a>>>(cb: CB) -> $Name<'a> {
                cb.into()
            }

            __callback_callfn! {
                $Name($($($a: $T),+)?)$(->$TReturn)?
            }

            pub fn is_nil(&self) -> bool {
                match self {
                    $Name::Nil => true,
                    _ => false
                }
            }
        }

        impl<'a> Default for $Name<'a> {
            fn default() -> $Name<'a> {
                $Name::Nil
            }
        }
    }
}

macro_rules! __callback_callfn {
    ($Name:ident($($($a:ident: $T:ty),+)?) -> $TReturn:ty) => {
        pub(crate) fn call(&mut self$(,$($a: $T),+)?) -> $TReturn {
            match self {
                $Name::CB(ref mut f) => f($($($a),+)?),
                $Name::Nil => Default::default(),
            }
        }
    };
    ($Name:ident($($($a:ident: $T:ty),+)?)) => {
        pub(crate) fn call(&mut self$(,$($a: $T),+)?) {
            match self {
                $Name::CB(ref mut f) => f($($($a),+)?),
                $Name::Nil => (),
            }
        }
    }
}

macro_rules! use_c_callback {
    ($ccb:expr, $cb:expr) => {
        if ($cb).is_nil() {
            None
        } else {
            Some($ccb as _)
        }
    };
}

#[cfg(not(windows))]
pub(crate) type NREAD = i64;
#[cfg(windows)]
pub(crate) type NREAD = isize;

pub type Result<T> = std::result::Result<T, Error>;

#[inline]
fn uvret(code: ::std::os::raw::c_int) -> Result<()> {
    if code < 0 {
        Err(Error::from_inner(code as uv::uv_errno_t))
    } else {
        Ok(())
    }
}

mod inner;
use inner::*;

pub mod error;
pub use error::Error::*;
pub use error::*;

pub mod version;
pub use version::*;

pub mod r#loop;
pub use r#loop::*;

pub mod buf;
pub use buf::*;

pub mod fs;
pub use fs::*;

pub mod net;
pub use net::*;

pub mod handles;
pub use handles::*;

pub mod requests;
pub use requests::*;

pub mod shared_libs;
pub use shared_libs::*;

pub mod misc;
pub use misc::*;

/// Imports some things that most every program will need.
pub mod prelude {
    pub use super::{
        BufTrait, Handle, HandleTrait, Loop, Req, ReqTrait, RunMode, StreamHandle, StreamTrait,
        ToStream,
    };
}
