use crate::{
	tools::{action_key::slot::SlotKey, bone::Bone},
	traits::{accessors::get::EntityContextMut, handles_skill_behaviors::SkillSpawner},
};
use bevy::ecs::system::SystemParam;
use std::{collections::HashMap, ops::DerefMut};

pub trait HandlesSKillControl {
	type TSkillControlMut<'w, 's>: SystemParam
		+ for<'c> EntityContextMut<SkillControl, TContext<'c>: HoldSkill>
		+ for<'c> EntityContextMut<SkillSpawnPoints, TContext<'c>: SpawnPointsDefinition>;
}

pub type SKillControlParamMut<'w, 's, T> = <T as HandlesSKillControl>::TSkillControlMut<'w, 's>;

pub struct SkillControl;

pub trait HoldSkill {
	/// Set this each frame
	fn holding<TSlot>(&mut self, key: TSlot)
	where
		TSlot: Into<SlotKey> + 'static;
}

impl<T> HoldSkill for T
where
	T: DerefMut<Target: HoldSkill>,
{
	fn holding<TSlot>(&mut self, key: TSlot)
	where
		TSlot: Into<SlotKey> + 'static,
	{
		self.deref_mut().holding(key);
	}
}

pub struct SkillSpawnPoints;

pub trait SpawnPointsDefinition {
	fn insert_spawn_point_definition(&mut self, definition: HashMap<Bone<'static>, SkillSpawner>);
}
