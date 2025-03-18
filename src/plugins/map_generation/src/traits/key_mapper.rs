use bevy::prelude::*;

pub trait KeyMapper {
	fn key_for(&self, translation: Vec3) -> Option<(usize, usize)>;
}
