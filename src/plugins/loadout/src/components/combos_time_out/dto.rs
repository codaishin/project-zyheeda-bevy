use crate::components::combos_time_out::CombosTimeOut;
use common::prelude::*;
use macros::serde_model;
use std::time::Duration;

#[serde_model]
#[derive(Debug, PartialEq)]
pub struct CombosTimeOutDto {
	max_duration: DurationInSeconds,
	duration: DurationInSeconds,
}

impl From<CombosTimeOut> for CombosTimeOutDto {
	fn from(
		CombosTimeOut {
			max_duration,
			duration,
		}: CombosTimeOut,
	) -> Self {
		Self {
			max_duration: DurationInSeconds::from(max_duration),
			duration: DurationInSeconds::from(duration),
		}
	}
}

impl TryLoadFrom<CombosTimeOutDto> for CombosTimeOut {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		CombosTimeOutDto {
			max_duration,
			duration,
		}: CombosTimeOutDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		Ok(Self {
			max_duration: Duration::from(max_duration),
			duration: Duration::from(duration),
		})
	}
}
