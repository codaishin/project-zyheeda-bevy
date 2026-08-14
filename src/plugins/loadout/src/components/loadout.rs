use crate::components::{
	active_skill::ActiveSkill,
	combos::CombosInternal,
	combos_time_out::CombosTimeOut,
	queue::Queue,
};
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	CombosInternal,
	CombosTimeOut = CombosTimeOut::after(Duration::from_secs(2)),
	Queue,
	ActiveSkill,
)]
pub(crate) struct Loadout;
