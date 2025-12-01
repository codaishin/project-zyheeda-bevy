use crate::components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	queue::Queue,
	skill_executer::SkillExecuter,
};
use bevy::prelude::*;
use std::{marker::PhantomData, time::Duration};

// FIXME: Remove generic type after dependency on agents has been removed and `AgentConfig` has
//        been reworked (limited to slot bone definitions)
#[derive(Component, Debug, PartialEq)]
#[require(
	Combos,
	CombosTimeOut = CombosTimeOut::after(Duration::from_secs(2)),
	Queue,
	SkillExecuter,
)]
pub(crate) struct Loadout<T = ()>(PhantomData<fn() -> T>);

impl<T> Default for Loadout<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}
