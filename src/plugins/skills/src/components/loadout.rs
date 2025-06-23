use crate::components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	queue::Queue,
	skill_executer::SkillExecuter,
	swapper::Swapper,
};
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component, Debug, Default)]
#[require(
	Combos,
	CombosTimeOut = CombosTimeOut::after(Duration::from_secs(2)),
	Queue,
	SkillExecuter,
	Swapper,
)]
pub(crate) struct Loadout;
