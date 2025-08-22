use crate::components::attacking::Attacking;
use common::{
	dto::duration_in_seconds::DurationInSeconds,
	errors::Unreachable,
	traits::handles_custom_assets::TryLoadFrom,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum AttackingDto {
	Hold {
		remaining: DurationInSeconds,
		cool_down: DurationInSeconds,
	},
	CoolDown {
		remaining: DurationInSeconds,
	},
}

impl From<Attacking> for AttackingDto {
	fn from(value: Attacking) -> Self {
		match value {
			Attacking::Hold {
				remaining,
				cool_down,
			} => Self::Hold {
				remaining: DurationInSeconds::from(remaining),
				cool_down: DurationInSeconds::from(cool_down),
			},
			Attacking::CoolDown { remaining } => Self::CoolDown {
				remaining: DurationInSeconds::from(remaining),
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
