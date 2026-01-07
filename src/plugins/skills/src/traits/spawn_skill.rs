mod extension;

use crate::{behaviors::skill_shape::OnSkillStop, traits::skill_builder::SkillLifetime};
use common::traits::handles_skill_physics::{
	Contact,
	Effect,
	Projection,
	SkillCaster,
	SkillSpawner,
	SkillTarget,
};

pub(crate) trait SpawnSkill<TSkillConfig> {
	fn spawn_skill(
		&mut self,
		config: TSkillConfig,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop;
}

pub(crate) trait SkillContact {
	fn skill_contact(
		&self,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> Contact;
}
pub(crate) trait SkillProjection {
	fn skill_projection(&self) -> Projection;
}

pub(crate) trait SkillContactEffects {
	type TIter<'a>: Iterator<Item = Effect>
	where
		Self: 'a;

	fn skill_contact_effects(&self) -> Self::TIter<'_>;
}

pub(crate) trait SkillProjectionEffects {
	type TIter<'a>: Iterator<Item = Effect>
	where
		Self: 'a;

	fn skill_projection_effects(&self) -> Self::TIter<'_>;
}

pub(crate) trait SkillData:
	SkillContact + SkillContactEffects + SkillProjection + SkillProjectionEffects + SkillLifetime
{
}

impl<T> SkillData for T where
	T: SkillContact
		+ SkillContactEffects
		+ SkillProjection
		+ SkillProjectionEffects
		+ SkillLifetime
{
}
