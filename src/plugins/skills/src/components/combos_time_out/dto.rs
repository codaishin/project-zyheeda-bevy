use crate::components::combos_time_out::CombosTimeOut;
use common::dto::duration::DurationDto;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CombosTimeOutDto {
	max_duration: DurationDto,
	duration: DurationDto,
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
			max_duration: DurationDto::from(max_duration),
			duration: DurationDto::from(duration),
		}
	}
}
