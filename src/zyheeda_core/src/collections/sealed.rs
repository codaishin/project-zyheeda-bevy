use std::collections::HashSet;

pub trait SealedCollection {}

impl<T, const N: usize> SealedCollection for [T; N] {}
impl<T> SealedCollection for [T] {}
impl<T> SealedCollection for Vec<T> {}
impl<T> SealedCollection for HashSet<T> {}
