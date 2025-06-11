use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct Bar {
	pub offset: Vec3,
	pub scale: f32,
	pub(crate) position: Option<Vec2>,
}

#[cfg(test)]
impl Bar {
	pub(crate) fn new(offset: Vec3, scale: f32) -> Self {
		Self {
			scale,
			offset,
			position: None,
		}
	}
}

impl Default for Bar {
	fn default() -> Self {
		Self {
			offset: Vec3::new(0., 2., 0.),
			scale: 1.,
			position: Default::default(),
		}
	}
}
