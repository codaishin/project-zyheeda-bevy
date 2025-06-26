use crate::components::movement::Movement;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, thread_safe::ThreadSafe},
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MovementDto<TMovement>
where
	TMovement: ThreadSafe + Default,
{
	target: Vec3,
	#[serde(skip)]
	_p: PhantomData<TMovement>,
}

impl<TMovement> From<Movement<TMovement>> for MovementDto<TMovement>
where
	TMovement: ThreadSafe + Default,
{
	fn from(Movement { target, .. }: Movement<TMovement>) -> Self {
		Self {
			target,
			_p: PhantomData,
		}
	}
}

impl<TMovement> TryLoadFrom<MovementDto<TMovement>> for Movement<TMovement>
where
	TMovement: ThreadSafe + Default,
{
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		MovementDto { target, .. }: MovementDto<TMovement>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self::to(target))
	}
}
