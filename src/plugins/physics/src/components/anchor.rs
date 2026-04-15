use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		accessors::get::View,
		handles_skill_physics::{SkillMount, SkillTarget},
	},
};

#[derive(Component, Debug, PartialEq)]
#[require(Transform, AnchorDirty)]
pub(crate) struct Anchor {
	pub(crate) attached_to: PersistentEntity,
	pub(crate) mount: SkillMount,
	pub(crate) rotation: AnchorRotation,
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
		self.rotation = AnchorRotation::OfAttachedTo;
		self
	}

	pub(crate) fn looking_at(mut self, target: SkillTarget) -> Self {
		self.rotation = AnchorRotation::LookingAt(target);
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
	pub(crate) fn on(self, mount: SkillMount) -> Anchor {
		Anchor {
			attached_to: self.attached_to,
			mount,
			rotation: AnchorRotation::OfMount,
			persistent: false,
		}
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum AnchorRotation {
	OfMount,
	OfAttachedTo,
	LookingAt(SkillTarget),
}
