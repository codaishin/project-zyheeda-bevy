pub(crate) mod shape_data;
pub(crate) mod start_behavior_data;

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
use shape_data::SkillShapeData;
use start_behavior_data::SkillBehaviorData;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillBehaviorConfigData<T> {
	shape: SkillShapeData<T>,
	contact: Vec<SkillBehaviorData>,
	projection: Vec<SkillBehaviorData>,
	spawn_on: SpawnOn,
}

impl<TLifeTimeIn, TLifeTimeOut> From<SkillBehaviorConfigData<TLifeTimeIn>>
	for SkillBehaviorConfig<TLifeTimeOut>
where
	LifeTimeDefinition: From<TLifeTimeOut>,
	TLifeTimeOut: Clone + From<TLifeTimeIn>,
{
	fn from(value: SkillBehaviorConfigData<TLifeTimeIn>) -> Self {
		let contact = value.contact.into_iter().map(SkillBehavior::from);
		let projection = value.projection.into_iter().map(SkillBehavior::from);

		Self::from_shape(BuildSkillShape::from(value.shape))
			.spawning_on(value.spawn_on)
			.with_contact_behaviors(contact.collect())
			.with_projection_behaviors(projection.collect())
	}
}
