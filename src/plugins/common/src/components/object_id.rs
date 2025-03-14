use bevy::prelude::*;
use uuid::Uuid;

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ObjectId(Uuid);

impl Default for ObjectId {
	fn default() -> Self {
		Self(Uuid::new_v4())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn default_is_not_nil() {
		let ObjectId(id) = ObjectId::default();

		assert!(id != Uuid::nil());
	}

	#[test]
	fn uuids_are_different() {
		let a = ObjectId::default();
		let b = ObjectId::default();

		assert!(
			a != b,
			"Expected left and right to be different, but they were equal\
			 \n  left: {a:?}\
			 \n right: {b:?}"
		);
	}
}
