use bevy::prelude::*;
use common::{tools::action_key::slot::SlotKey, traits::handles_skill_behaviors::HoldSkills};
use std::{
	collections::{HashSet, hash_set::Iter},
	iter::Cloned,
};

#[derive(Component, Debug, PartialEq, Default)]
pub struct SkillUsage {
	pub(crate) holding: HashSet<SlotKey>,
	pub(crate) started_holding: HashSet<SlotKey>,
}

impl HoldSkills for SkillUsage {
	type Iter<'a> = Cloned<Iter<'a, SlotKey>>;

	fn holding(&self) -> Self::Iter<'_> {
		self.holding.iter().cloned()
	}

	fn started_holding(&self) -> Self::Iter<'_> {
		self.started_holding.iter().cloned()
	}
}
