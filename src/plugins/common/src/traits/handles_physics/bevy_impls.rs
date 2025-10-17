//! Implementations for common bevy system parameters to forward traits
//! from the inner to the outer type.

use crate::traits::handles_physics::{Raycast, RaycastExtra};
use bevy::prelude::*;
use std::ops::Deref;

impl<T, TExtra> Raycast<TExtra> for Res<'_, T>
where
	T: Resource + Raycast<TExtra>,
	TExtra: RaycastExtra,
{
	fn raycast(&self, ray: Ray3d, constraints: TExtra) -> TExtra::TResult {
		self.deref().raycast(ray, constraints)
	}
}
