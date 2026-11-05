use bevy::prelude::*;
use std::ops::{Deref, DerefMut, Not};

#[derive(Resource, Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct DropdownChanged(pub(crate) bool);

impl Deref for DropdownChanged {
	type Target = bool;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for DropdownChanged {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Not for DropdownChanged {
	type Output = bool;

	fn not(self) -> Self::Output {
		!self.0
	}
}

impl Not for &DropdownChanged {
	type Output = bool;

	fn not(self) -> Self::Output {
		!self.0
	}
}
