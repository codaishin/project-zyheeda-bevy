use crate::behaviors::{
	build_skill_shape::OnSkillStop,
	spawn_on::SpawnOn,
	SkillCaster,
	SkillSpawner,
	Target,
};
use bevy::prelude::*;
use std::time::Duration;

pub(crate) trait SpawnSkillBehavior<TCommands> {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TLifetime>(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop
	where
		TLifetime: From<Duration> + Component;
}
