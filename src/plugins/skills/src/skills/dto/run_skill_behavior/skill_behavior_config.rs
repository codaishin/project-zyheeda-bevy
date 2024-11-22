pub(crate) mod shape;
pub(crate) mod start_behavior;

use crate::{
	behaviors::{
		build_skill_shape::BuildSkillShape,
		spawn_on::SpawnOn,
		start_behavior::SkillBehavior,
		SkillBehaviorConfig,
	},
	skills::lifetime::LifeTimeDefinition,
};
use serde::{Deserialize, Serialize};
use shape::SkillShapeDto;
use start_behavior::SkillBehaviorDto;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillBehaviorConfigDto<T> {
	shape: SkillShapeDto<T>,
	contact: Vec<SkillBehaviorDto>,
	projection: Vec<SkillBehaviorDto>,
	spawn_on: SpawnOn,
}

impl<TLifeTimeIn, TLifeTimeOut> From<SkillBehaviorConfigDto<TLifeTimeIn>>
	for SkillBehaviorConfig<TLifeTimeOut>
where
	LifeTimeDefinition: From<TLifeTimeOut>,
	TLifeTimeOut: Clone + From<TLifeTimeIn>,
{
	fn from(value: SkillBehaviorConfigDto<TLifeTimeIn>) -> Self {
		let contact = value.contact.into_iter().map(SkillBehavior::from);
		let projection = value.projection.into_iter().map(SkillBehavior::from);

		Self::from_shape(BuildSkillShape::from(value.shape))
			.spawning_on(value.spawn_on)
			.with_contact_behaviors(contact.collect())
			.with_projection_behaviors(projection.collect())
	}
}
