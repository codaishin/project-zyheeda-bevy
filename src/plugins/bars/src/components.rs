use bevy::{
	ecs::{component::Component, entity::Entity},
	math::{Vec2, Vec3},
};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UI {
	pub background: Entity,
	pub foreground: Entity,
}

#[derive(Component)]
pub(crate) struct BarValues<T> {
	pub current: f32,
	pub max: f32,
	pub ui: Option<UI>,
	pub phantom_data: PhantomData<T>,
}

#[cfg(test)]
impl<T> BarValues<T> {
	pub fn new(current: f32, max: f32) -> Self {
		Self {
			current,
			max,
			ui: None,
			phantom_data: PhantomData,
		}
	}
}

impl<T> Default for BarValues<T> {
	fn default() -> Self {
		Self {
			ui: Default::default(),
			current: Default::default(),
			max: Default::default(),
			phantom_data: Default::default(),
		}
	}
}

#[derive(Component)]
pub struct Bar {
	pub offset: Vec3,
	pub scale: f32,
	pub(crate) position: Option<Vec2>,
}

impl Bar {
	pub fn new(offset: Vec3, scale: f32) -> Self {
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
