pub mod ordered_hash_map;

use crate::{
	components::Player,
	traits::{clamp_zero_positive::ClampZeroPositive, load_asset::Path},
};
use bevy::prelude::Entity;
use macros::ClampZeroPositive;
use serde::{Deserialize, Serialize};
use std::{
	fmt::{Debug, Formatter, Result},
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

///Serves as a struct to implement static traits on
pub struct Tools;

pub struct Factory<T>(PhantomData<T>);

#[derive(Debug, PartialEq)]
pub struct This<'a, T: Debug + PartialEq>(pub &'a mut T);

impl<'a, T: Debug + PartialEq> Deref for This<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<'a, T: Debug + PartialEq> DerefMut for This<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0
	}
}

#[derive(Debug, PartialEq)]
pub struct Last<'a, T: Debug + PartialEq>(pub &'a T);

impl<'a, T: Debug + PartialEq> Deref for Last<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, ClampZeroPositive)]
pub struct UnitsPerSecond(f32);

#[derive(Debug, PartialEq, Clone, Copy, ClampZeroPositive, Serialize, Deserialize)]
pub struct Units(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct Intensity(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct IntensityChangePerSecond(f32);

pub fn player_animation_path(animation_name: &str) -> Path {
	Path::from(Player::MODEL_PATH.to_owned() + "#" + animation_name)
}

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
