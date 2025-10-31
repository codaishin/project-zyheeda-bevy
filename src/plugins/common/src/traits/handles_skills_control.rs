use crate::{
	tools::{action_key::slot::SlotKey, bone::Bone},
	traits::{accessors::get::EntityContextMut, handles_skill_behaviors::SkillSpawner},
};
use bevy::ecs::system::SystemParam;
use std::collections::HashMap;

pub trait HandlesSKillControl {
	type TSkillControlMut<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<SkillControl, TContext<'c>: HoldSkill>
		+ for<'c> EntityContextMut<SkillSpawnPoints, TContext<'c>: SpawnPointsDefinition>;
}

pub struct SkillControl;

pub trait HoldSkill {
	/// Set this each frame
	fn holding<TSlot>(&mut self, key: TSlot)
	where
		TSlot: Into<SlotKey>;
}

pub struct SkillSpawnPoints;

pub trait SpawnPointsDefinition {
	fn insert_spawn_point_definition(&mut self, definition: HashMap<Bone<'static>, SkillSpawner>);
}
