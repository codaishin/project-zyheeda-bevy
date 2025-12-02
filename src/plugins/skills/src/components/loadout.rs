use crate::components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	queue::Queue,
	skill_executer::SkillExecuter,
};
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	Combos,
	CombosTimeOut = CombosTimeOut::after(Duration::from_secs(2)),
	Queue,
	SkillExecuter,
)]
pub(crate) struct Loadout;
