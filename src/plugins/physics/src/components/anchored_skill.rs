use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, AnchoredSkillDirty)]
pub(crate) struct AnchoredSkill {
	pub(crate) attached_to: PersistentEntity,
	pub(crate) mount: SkillMount,
	pub(crate) rotation: AnchorRotation,
	pub(crate) persistent: bool,
}

impl AnchoredSkill {
	pub(crate) fn to<TEntity>(entity: TEntity) -> AnchoredSkillAttachment
	where
		TEntity: Into<PersistentEntity>,
	{
		AnchoredSkillAttachment {
			attached_to: entity.into(),
		}
	}

	pub(crate) fn with_attached_rotation(mut self) -> Self {
		self.rotation = AnchorRotation::OfAttachedTo;
		self
	}

	pub(crate) fn looking_at_skill_target(mut self) -> Self {
		self.rotation = AnchorRotation::LookingAtSkillTarget;
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

impl View<PersistentEntity> for AnchoredSkill {
	fn view(&self) -> PersistentEntity {
		self.attached_to
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub(crate) struct AnchoredSkillDirty;

pub(crate) struct AnchoredSkillAttachment {
	attached_to: PersistentEntity,
}

impl AnchoredSkillAttachment {
	pub(crate) fn on(self, mount: SkillMount) -> AnchoredSkill {
		AnchoredSkill {
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
	LookingAtSkillTarget,
}
