use core::mem::discriminant;
use std::fmt::{Debug, Formatter, Result};

pub enum Changed<T> {
	Value(T),
	None,
}

impl<T: Debug> Debug for Changed<T> {
	fn fmt(&self, formatter: &mut Formatter) -> Result {
		match self {
			Self::Value(value) => formatter.debug_tuple("Value").field(value).finish(),
			Self::None => write!(formatter, "None"),
		}
	}
}

impl<T: PartialEq> PartialEq for Changed<T> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Value(l0), Self::Value(r0)) => l0 == r0,
			_ => discriminant(self) == discriminant(other),
		}
	}
}

impl<T: Clone> Clone for Changed<T> {
	fn clone(&self) -> Self {
		match self {
			Self::Value(value) => Self::Value(value.clone()),
			Self::None => Self::None,
		}
	}
}
