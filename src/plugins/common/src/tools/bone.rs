use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Bone(pub &'static str);

impl Deref for Bone {
	type Target = &'static str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
