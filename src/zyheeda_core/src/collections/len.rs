mod array;
mod hash_set;
mod slice;
mod vec;

use crate::collections::is_empty::IsEmpty;

pub trait Len: IsEmpty {
	fn len(&self) -> usize;
}
