use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{accessors::get::View, handles_skill_physics::SkillSpawner},
};

#[derive(Component, Debug, PartialEq)]
#[require(Transform, AnchorDirty)]
pub(crate) struct Anchor {
	pub(crate) attached_to: PersistentEntity,
	pub(crate) attach_point: SkillSpawner,
	pub(crate) use_attached_rotation: bool,
	pub(crate) persistent: bool,
}

impl Anchor {
	pub(crate) fn attach_to<TEntity>(entity: TEntity) -> AnchorAttachment
	where
		TEntity: Into<PersistentEntity>,
	{
		AnchorAttachment {
			attached_to: entity.into(),
		}
	}

	pub(crate) fn with_attached_rotation(mut self) -> Self {
		self.use_attached_rotation = true;
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
		self.attached_to
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub(crate) struct AnchorDirty;

pub(crate) struct AnchorAttachment {
	attached_to: PersistentEntity,
}

impl AnchorAttachment {
	pub(crate) fn on(self, attach_point: SkillSpawner) -> Anchor {
		Anchor {
			attached_to: self.attached_to,
			attach_point,
			use_attached_rotation: false,
			persistent: false,
		}
	}
}
