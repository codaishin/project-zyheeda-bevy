use crate::components::movement::Movement;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_movement_behavior::PathMotionSpec,
		thread_safe::ThreadSafe,
	},
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MovementDto<TMovement>
where
	TMovement: ThreadSafe,
{
	target: PathMotionSpec,
	#[serde(skip)]
	_m: PhantomData<TMovement>,
}

impl<TMovement> From<Movement<TMovement>> for MovementDto<TMovement>
where
	TMovement: ThreadSafe,
{
	fn from(Movement { spec: target, .. }: Movement<TMovement>) -> Self {
		Self {
			target,
			_m: PhantomData,
		}
	}
}

impl<TMovement> TryLoadFrom<MovementDto<TMovement>> for Movement<TMovement>
where
	TMovement: ThreadSafe,
{
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		MovementDto { target, .. }: MovementDto<TMovement>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self {
			spec: target,
			_m: PhantomData,
		})
	}
}
