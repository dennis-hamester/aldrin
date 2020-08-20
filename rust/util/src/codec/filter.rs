use bytes::BytesMut;
use std::error::Error as StdError;
use std::fmt;

pub trait Filter {
    type Error;

    fn forward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error>;
    fn backward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error>;
}

impl<T: Filter + ?Sized> Filter for &mut T {
    type Error = T::Error;

    fn forward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        (*self).forward(data)
    }

    fn backward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        (*self).backward(data)
    }
}

impl<T: Filter + ?Sized> Filter for Box<T> {
    type Error = T::Error;

    fn forward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        (**self).forward(data)
    }

    fn backward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        (**self).backward(data)
    }
}

pub trait FilterExt: Filter {
    fn map_err<F, E>(self, f: F) -> MapError<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapError {
            map_err: f,
            filter: self,
        }
    }

    fn chain<T>(self, next: T) -> Chain<Self, T>
    where
        Self: Sized,
    {
        Chain { f1: self, f2: next }
    }
}

impl<T: Filter + ?Sized> FilterExt for T {}

#[derive(Debug)]
pub struct MapError<T: ?Sized, F> {
    map_err: F,
    filter: T,
}

impl<T, F, E> Filter for MapError<T, F>
where
    T: Filter + ?Sized,
    F: FnMut(T::Error) -> E,
{
    type Error = E;

    fn forward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        self.filter.forward(data).map_err(&mut self.map_err)
    }

    fn backward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        self.filter.backward(data).map_err(&mut self.map_err)
    }
}

#[derive(Debug)]
pub struct Chain<F1, F2> {
    f1: F1,
    f2: F2,
}

impl<F1, F2> Filter for Chain<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    type Error = ChainError<F1::Error, F2::Error>;

    fn forward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        let data = self.f1.forward(data).map_err(ChainError::Filter1)?;
        self.f2.forward(data).map_err(ChainError::Filter2)
    }

    fn backward(&mut self, data: BytesMut) -> Result<BytesMut, Self::Error> {
        let data = self.f2.backward(data).map_err(ChainError::Filter2)?;
        self.f1.backward(data).map_err(ChainError::Filter1)
    }
}

#[derive(Debug)]
pub enum ChainError<E1, E2> {
    Filter1(E1),
    Filter2(E2),
}

impl<E1, E2> fmt::Display for ChainError<E1, E2>
where
    E1: fmt::Display,
    E2: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChainError::Filter1(e) => e.fmt(f),
            ChainError::Filter2(e) => e.fmt(f),
        }
    }
}

impl<E1, E2> StdError for ChainError<E1, E2>
where
    E1: fmt::Display + fmt::Debug,
    E2: fmt::Display + fmt::Debug,
{
}
