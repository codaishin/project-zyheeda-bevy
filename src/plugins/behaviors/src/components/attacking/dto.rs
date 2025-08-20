use crate::components::attacking::Attacking;
use common::{
	dto::duration_secs_f32::DurationSecsF32,
	errors::Unreachable,
	traits::handles_custom_assets::TryLoadFrom,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AttackingDto {
	Hold {
		remaining: DurationSecsF32,
		cool_down: DurationSecsF32,
	},
	CoolDown {
		remaining: DurationSecsF32,
	},
}

impl From<Attacking> for AttackingDto {
	fn from(value: Attacking) -> Self {
		match value {
			Attacking::Hold {
				remaining,
				cool_down,
			} => Self::Hold {
				remaining: DurationSecsF32::from(remaining),
				cool_down: DurationSecsF32::from(cool_down),
			},
			Attacking::CoolDown { remaining } => Self::CoolDown {
				remaining: DurationSecsF32::from(remaining),
			},
		}
	}
}

impl TryLoadFrom<AttackingDto> for Attacking {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		from: AttackingDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(match from {
			AttackingDto::Hold {
				remaining,
				cool_down,
			} => Self::Hold {
				remaining: Duration::from(remaining),
				cool_down: Duration::from(cool_down),
			},
			AttackingDto::CoolDown { remaining } => Self::CoolDown {
				remaining: Duration::from(remaining),
			},
		})
	}
}
