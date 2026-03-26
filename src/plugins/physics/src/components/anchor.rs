use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::View, handles_skill_physics::SkillSpawner},
};

#[derive(Component, Debug, PartialEq)]
#[require(Transform, AnchorDirty)]
pub(crate) struct Anchor {
	pub(crate) target: PersistentEntity,
	pub(crate) skill_spawner: SkillSpawner,
	pub(crate) use_target_rotation: bool,
	pub(crate) persistent: bool,
}

impl Anchor {
	pub(crate) fn to_target<TEntity>(target: TEntity) -> AnchorTarget
	where
		TEntity: Into<PersistentEntity>,
	{
		AnchorTarget {
			target: target.into(),
		}
	}

	pub(crate) fn with_target_rotation(mut self) -> Self {
		self.use_target_rotation = true;
		self
	}

	pub(crate) fn once(mut self) -> Self {
		self.persistent = false;
		self
	}

	pub(crate) fn always(mut self) -> Self {
		self.persistent = true;
		self
	}
}

impl View<PersistentEntity> for Anchor {
	fn view(&self) -> PersistentEntity {
		self.target
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub(crate) struct AnchorDirty;

pub(crate) struct AnchorTarget {
	target: PersistentEntity,
}

impl AnchorTarget {
	pub(crate) fn on_spawner(self, spawner: SkillSpawner) -> Anchor {
		Anchor {
			target: self.target,
			skill_spawner: spawner,
			use_target_rotation: false,
			persistent: false,
		}
	}
}
