use bevy::prelude::*;

pub(crate) trait KeyMapper {
	fn key_for(&self, translation: Vec3) -> (i32, i32);
}
