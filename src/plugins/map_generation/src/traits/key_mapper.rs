use bevy::prelude::*;

pub trait KeyMapper {
	fn key_for(&self, translation: Vec3) -> (i32, i32);
}
