use crate::components::combos_time_out::CombosTimeOut;
use common::dto::duration_secs_f32::DurationSecsF32;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CombosTimeOutDto {
	max_duration: DurationSecsF32,
	duration: DurationSecsF32,
}

impl From<CombosTimeOutDto> for CombosTimeOut {
	fn from(
		CombosTimeOutDto {
			max_duration,
			duration,
		}: CombosTimeOutDto,
	) -> Self {
		Self {
			max_duration: Duration::from(max_duration),
			duration: Duration::from(duration),
		}
	}
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
