//! Implementations for common bevy system parameters to forward traits
//! from the inner to the outer type.

use crate::traits::handles_physics::{Raycast, RaycastExtra};
use bevy::prelude::*;
use std::ops::DerefMut;

impl<T, TArgs> Raycast<TArgs> for ResMut<'_, T>
where
	T: Resource + Raycast<TArgs>,
	TArgs: RaycastExtra,
{
	fn raycast(&mut self, args: TArgs) -> TArgs::TResult {
		self.deref_mut().raycast(args)
	}
}
