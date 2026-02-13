use crate::components::movement::Movement;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, handles_movement::MovementTarget},
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MovementDto {
	#[serde(skip_serializing_if = "Option::is_none")]
	target: Option<MovementTarget>,
}

impl<TMovement> From<Movement<TMovement>> for MovementDto {
	fn from(Movement { target, .. }: Movement<TMovement>) -> Self {
		Self { target }
	}
}

impl<TMovement> TryLoadFrom<MovementDto> for Movement<TMovement> {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		MovementDto { target, .. }: MovementDto,
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
	#![allow(clippy::unwrap_used)]
	use super::*;

	#[test]
	fn round_trip_empty_target() {
		let obj = MovementDto { target: None };

		let json = serde_json::to_string(&obj).unwrap();
		let deserialized = serde_json::from_str::<MovementDto>(&json).unwrap();

		assert_eq!(obj, deserialized);
	}

	#[test]
	fn round_trip_with_target() {
		let obj = MovementDto {
			target: Some(MovementTarget::Dir(Dir3::NEG_X)),
		};

		let json = serde_json::to_string(&obj).unwrap();
		let deserialized = serde_json::from_str::<MovementDto>(&json).unwrap();

		assert_eq!(obj, deserialized);
	}
}
