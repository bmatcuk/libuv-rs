//! Internal utilities

/// An internal version of From<T>
#[doc(hidden)]
pub trait FromInner<T>: Sized {
    fn from_inner(_: T) -> Self;
}

/// An internal version of Into<T>
#[doc(hidden)]
pub trait IntoInner<T>: Sized {
    fn into_inner(self) -> T;
}

/// An internal version of TryFrom<T>
#[doc(hidden)]
pub trait TryFromInner<T>: Sized {
    type Error;
    fn try_from_inner(_: T) -> Result<Self, Self::Error>;
}

/// An internal version of TryInto<T>
#[doc(hidden)]
pub trait TryIntoInner<T>: Sized {
    type Error;
    fn try_into_inner(self) -> Result<T, Self::Error>;
}

// FromInner implies IntoInner
impl<T, U> IntoInner<U> for T where U: FromInner<T> {
    fn into_inner(self) -> U {
        U::from_inner(self)
    }
}

// FromInner (and thus IntoInner) is reflexive
impl<T> FromInner<T> for T {
    fn from_inner(t: T) -> T { t }
}

// TryFromInner implies TryIntoInner
impl<T, U> TryIntoInner<U> for T where U: TryFromInner<T> {
    type Error = U::Error;

    fn try_into_inner(self) -> Result<U, U::Error> {
        U::try_from_inner(self)
    }
}

// Infallible conversions are semantically equivalent to fallible conversions with an unihabited
// error type
impl<T, U> TryFromInner<U> for T where U: IntoInner<T> {
    type Error = std::convert::Infallible;

    fn try_from_inner(value: U) -> Result<Self, Self::Error> {
        Ok(U::into_inner(value))
    }
}
