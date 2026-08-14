use crate::traits::iteration::{FiniteIter, IterFinite};

/// Get the first element of an [`IterFinite`] type and wrap it in a container
///
/// # Example
/// ```
/// use common::prelude::*;
///
/// #[derive(Debug, PartialEq)]
/// struct Container(State);
///
/// #[derive(Debug, PartialEq, Clone, Copy)]
/// enum State { A, B }
///
/// impl IterFinite for State {
///   fn iterator() -> FiniteIter<Self> {
///     FiniteIter(Some(Self::A))
///   }
///
///   fn next(current: &FiniteIter<Self>) -> Option<Self> {
///     match current.0? {
///       Self::A => Some(Self::B),
///       Self::B => None,
///     }
///   }
/// }
///
/// assert_eq!(Some(Container(State::A)), first(Container));
/// ```
pub fn first<TOuter, TInner>(wrap: impl Fn(TInner) -> TOuter) -> Option<TOuter>
where
	TInner: IterFinite,
{
	TInner::iterator().0.map(wrap)
}

/// Get the next element of an [`IterFinite`] type and wrap it in a container
///
/// # Example
/// ```
/// use common::prelude::*;
///
/// #[derive(Debug, PartialEq)]
/// struct Container(State);
///
/// #[derive(Debug, PartialEq, Clone, Copy)]
/// enum State { A, B }
///
/// impl IterFinite for State {
///   fn iterator() -> FiniteIter<Self> {
///     FiniteIter(Some(Self::A))
///   }
///
///   fn next(current: &FiniteIter<Self>) -> Option<Self> {
///     match current.0? {
///       Self::A => Some(Self::B),
///       Self::B => None,
///     }
///   }
/// }
///
/// assert_eq!(Some(Container(State::B)), next(Container, State::A));
/// ```
pub fn next<TOuter, TInner>(wrap: impl Fn(TInner) -> TOuter, key: TInner) -> Option<TOuter>
where
	TInner: IterFinite,
{
	TInner::next(&FiniteIter(Some(key))).map(wrap)
}
