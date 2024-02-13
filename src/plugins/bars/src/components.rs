use bevy::{
	ecs::{component::Component, entity::Entity},
	math::Vec2,
};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UI {
	pub background: Entity,
	pub foreground: Entity,
}

#[derive(Component)]
pub struct Bar<T> {
	pub position: Option<Vec2>,
	pub ui: Option<UI>,
	pub current: f32,
	pub max: f32,
	pub scale: f32,
	pub phantom_data: PhantomData<T>,
}

impl<T> Bar<T> {
	pub fn new(position: Option<Vec2>, current: f32, max: f32, scale: f32) -> Self {
		Self {
			position,
			ui: None,
			current,
			max,
			scale,
			phantom_data: PhantomData,
		}
	}
}
