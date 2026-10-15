mod array;
mod hash_set;
mod slice;
mod vec;

use crate::collections::sealed::SealedCollection;

pub trait IsEmpty: SealedCollection {
	fn is_empty(&self) -> bool;
}
