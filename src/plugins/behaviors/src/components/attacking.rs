mod dto;

use crate::{components::attacking::dto::AttackingDto, traits::count_down::CountDown};
use bevy::prelude::*;
use macros::SavableComponent;
use std::time::Duration;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy)]
#[savable_component(dto = AttackingDto)]
pub(crate) enum Attacking {
	Hold {
		remaining: Duration,
		cool_down: Duration,
	},
	CoolDown {
		remaining: Duration,
	},
}

impl CountDown for Attacking {
	fn remaining_mut(&mut self) -> &mut Duration {
		match self {
			Attacking::Hold { remaining, .. } => remaining,
			Attacking::CoolDown { remaining, .. } => remaining,
		}
	}

	fn next_state(&self) -> Option<Self> {
		match self {
			Attacking::Hold { cool_down, .. } => Some(Self::CoolDown {
				remaining: *cool_down,
			}),
			Attacking::CoolDown { .. } => None,
		}
	}
}
