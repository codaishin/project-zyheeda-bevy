use crate::components::ui::UI;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component)]
pub(crate) struct BarValues<T> {
	pub current: f32,
	pub max: f32,
	pub ui: Option<UI>,
	phantom_data: PhantomData<T>,
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
