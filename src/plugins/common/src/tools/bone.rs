use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Bone<'a>(pub &'a str);

impl<'a> Deref for Bone<'a> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}
