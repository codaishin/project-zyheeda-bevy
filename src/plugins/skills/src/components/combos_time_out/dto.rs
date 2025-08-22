use crate::components::combos_time_out::CombosTimeOut;
use common::{
	dto::duration_in_seconds::DurationInSeconds,
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
