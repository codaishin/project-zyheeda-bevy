pub mod action_key;
pub mod aggro_range;
pub mod animation_key;
pub mod attack_range;
pub mod bone;
pub mod change;
pub mod collider_info;
pub mod collider_radius;
pub mod handle;
pub mod inventory_key;
pub mod item_type;
pub mod iter_helpers;
pub mod movement_animation;
pub mod ordered_hash_map;
pub mod skill_execution;
pub mod speed;
pub mod swap_key;

pub(crate) mod get_recursively;

use bevy::prelude::Entity;
use macros::ClampZeroPositive;
use serde::Serialize;
use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

#[derive(Debug, PartialEq)]
pub struct This<'a, T: Debug + PartialEq>(pub &'a mut T);

impl<T: Debug + PartialEq> Deref for This<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<T: Debug + PartialEq> DerefMut for This<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0
	}
}

#[derive(Debug, PartialEq)]
pub struct Last<'a, T: Debug + PartialEq>(pub &'a T);

impl<T: Debug + PartialEq> Deref for Last<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, ClampZeroPositive, Serialize)]
pub struct UnitsPerSecond(f32);

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, ClampZeroPositive, Serialize)]
pub struct Units(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct Intensity(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct IntensityChangePerSecond(f32);

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Index<T>(pub T);

#[derive(Default, Clone, Debug, PartialEq)]
pub enum Focus {
	#[default]
	Unchanged,
	New(Vec<Entity>),
}

impl From<Vec<Entity>> for Focus {
	fn from(entities: Vec<Entity>) -> Self {
		Focus::New(entities)
	}
}

#[cfg(test)]
mod test_clamp_zero_positive {
	use super::*;
	use serde_json::json;

	#[derive(ClampZeroPositive, Debug, PartialEq)]
	struct _Value(f32);

	#[test]
	fn set_value() {
		assert_eq!(&42., _Value::from(42.).deref());
	}

	#[test]
	fn min_zero() {
		assert_eq!(&0., _Value::from(-42.).deref());
	}

	#[test]
	fn zero_from_nan() {
		assert_eq!(&0., _Value::from(f32::NAN).deref());
	}

	#[test]
	fn deserialize_value() {
		let json = json!(42);

		let value = serde_json::from_value::<_Value>(json);

		assert_eq!(_Value(42.), value.unwrap());
	}

	#[test]
	fn deserialize_min_zero() {
		let json = json!(-42);

		let value = serde_json::from_value::<_Value>(json);

		assert_eq!(_Value(0.), value.unwrap());
	}
}
