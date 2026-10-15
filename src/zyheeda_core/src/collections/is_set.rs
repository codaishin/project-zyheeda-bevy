use crate::collections::{
	is_empty::IsEmpty,
	len::Len,
	ordered::OrderedSet,
	sealed::SealedCollection,
};
use std::collections::HashSet;

/// Marks containers where each item is unique:
/// - [`HashSet`](`std::collections::hash_set::HashSet`)
/// - [`OrderedSet`](`crate::collections::ordered::OrderedSet`)
pub trait IsSet: SealedCollection + IsEmpty + Len + IntoIterator {}

impl<T> IsSet for HashSet<T> {}
impl<T> IsSet for OrderedSet<T> where T: PartialEq {}
