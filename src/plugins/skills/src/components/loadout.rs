use crate::components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	queue::Queue,
	skill_executer::SkillExecuter,
};
use bevy::prelude::*;
use common::traits::{loadout::LoadoutConfig, thread_safe::ThreadSafe};
use std::{marker::PhantomData, time::Duration};

#[derive(Component, Debug, PartialEq)]
#[require(
	Combos,
	CombosTimeOut = CombosTimeOut::after(Duration::from_secs(2)),
	Queue,
	SkillExecuter,
)]
pub(crate) struct Loadout<T>(PhantomData<T>)
where
	T: LoadoutConfig + ThreadSafe;

impl<T> Default for Loadout<T>
where
	T: LoadoutConfig + ThreadSafe,
{
	fn default() -> Self {
		Self(PhantomData)
	}
}
