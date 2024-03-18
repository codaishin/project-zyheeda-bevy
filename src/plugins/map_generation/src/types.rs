use std::ops::Deref;

pub(crate) struct ForChildren(bool);

impl Deref for ForChildren {
	type Target = bool;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<bool> for ForChildren {
	fn from(value: bool) -> Self {
		Self(value)
	}
}
