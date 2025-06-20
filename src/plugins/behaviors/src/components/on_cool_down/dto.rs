use crate::components::on_cool_down::OnCoolDown;
use common::{
	dto::duration_secs_f32::DurationSecsF32,
	errors::Unreachable,
	traits::handles_custom_assets::TryLoadFrom,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct OnCoolDownDto(DurationSecsF32);

impl From<OnCoolDown> for OnCoolDownDto {
	fn from(OnCoolDown(cool_down): OnCoolDown) -> Self {
		Self(DurationSecsF32::from(cool_down))
	}
}

impl TryLoadFrom<OnCoolDownDto> for OnCoolDown {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		OnCoolDownDto(cool_down): OnCoolDownDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self(Duration::from(cool_down)))
	}
}
