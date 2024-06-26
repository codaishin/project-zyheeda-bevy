pub mod changed;

use crate::{
	components::Player,
	traits::{clamp_zero_positive::ClampZeroPositive, load_asset::Path},
};
use macros::ClampZeroPositive;
use std::{
	fmt::Debug,
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

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Default, ClampZeroPositive)]
pub struct UnitsPerSecond(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct Units(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct Intensity(f32);

#[derive(Debug, PartialEq, Clone, ClampZeroPositive)]
pub struct IntensityChangePerSecond(f32);

pub fn player_animation_path(animation_name: &str) -> Path {
	Path::from(Player::MODEL_PATH.to_owned() + "#" + animation_name)
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
