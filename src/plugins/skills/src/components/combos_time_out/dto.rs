use crate::components::combos_time_out::CombosTimeOut;
use common::{
	dto::duration_secs_f32::DurationSecsF32,
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CombosTimeOutDto {
	max_duration: DurationSecsF32,
	duration: DurationSecsF32,
}

impl From<CombosTimeOut> for CombosTimeOutDto {
	fn from(
		CombosTimeOut {
			max_duration,
			duration,
		}: CombosTimeOut,
	) -> Self {
		Self {
			max_duration: DurationSecsF32::from(max_duration),
			duration: DurationSecsF32::from(duration),
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
