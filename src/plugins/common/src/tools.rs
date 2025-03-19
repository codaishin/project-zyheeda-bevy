pub mod aggro_range;
pub mod attack_range;
pub mod change;
pub mod collider_info;
pub mod collider_radius;
pub mod exclude_rigid_body;
pub mod handle;
pub mod inventory_key;
pub mod item_description;
pub mod item_type;
pub mod movement_animation;
pub mod ordered_hash_map;
pub mod skill_description;
pub mod skill_execution;
pub mod skill_icon;
pub mod slot_key;
pub mod speed;
pub mod swap_key;

pub(crate) mod get_recursively;

use crate::traits::clamp_zero_positive::ClampZeroPositive;
use bevy::prelude::Entity;
use macros::ClampZeroPositive;
use serde::{Deserialize, Serialize};
use std::{
	fmt::{Debug, Formatter, Result},
	ops::{Deref, DerefMut},
};

///Serves as a struct to implement static traits on
pub struct Tools;

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

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, ClampZeroPositive, Serialize, Deserialize)]
pub struct UnitsPerSecond(f32);

#[derive(Debug, PartialEq, Clone, Copy, ClampZeroPositive, Serialize, Deserialize, PartialOrd)]
pub struct Units(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct Intensity(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct IntensityChangePerSecond(f32);

#[derive(Default)]
pub struct Index<T>(pub T);

impl<T: Debug> Debug for Index<T> {
	fn fmt(&self, formatter: &mut Formatter) -> Result {
		formatter.debug_tuple("Index").field(&self.0).finish()
	}
}

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

	#[derive(ClampZeroPositive)]
	struct _Value(f32);

	#[test]
	fn set_value() {
		assert_eq!(&42., _Value::new(42.).deref());
	}

	#[test]
	fn min_zero() {
		assert_eq!(&0., _Value::new(-42.).deref());
	}
}
