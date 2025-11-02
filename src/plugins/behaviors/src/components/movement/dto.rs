use crate::components::movement::Movement;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_movement::MovementTarget,
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
	#[serde(skip_serializing_if = "Option::is_none")]
	target: Option<MovementTarget>,
	#[serde(skip)]
	_m: PhantomData<TMovement>,
}

impl<TMovement> From<Movement<TMovement>> for MovementDto<TMovement>
where
	TMovement: ThreadSafe,
{
	fn from(Movement { target, .. }: Movement<TMovement>) -> Self {
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
			target,
			_m: PhantomData,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::marker::PhantomData;

	#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
	struct _Method;

	#[test]
	fn round_trip_empty_target() {
		let obj = MovementDto::<_Method> {
			target: None,
			_m: PhantomData,
		};

		let json = serde_json::to_string(&obj).unwrap();
		let deserialized = serde_json::from_str::<MovementDto<_Method>>(&json).unwrap();

		assert_eq!(obj, deserialized);
	}

	#[test]
	fn round_trip_with_target() {
		let obj = MovementDto::<_Method> {
			target: Some(MovementTarget::Dir(Dir3::NEG_X)),
			_m: PhantomData,
		};

		let json = serde_json::to_string(&obj).unwrap();
		let deserialized = serde_json::from_str::<MovementDto<_Method>>(&json).unwrap();

		assert_eq!(obj, deserialized);
	}
}
