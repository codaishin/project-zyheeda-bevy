pub(crate) mod shape_data;
pub(crate) mod start_behavior_data;

use crate::{
	behaviors::{
		build_skill_shape::BuildSkillShape,
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
}

impl<TLifeTime> From<SkillBehaviorConfigData<TLifeTime>> for SkillBehaviorConfig<TLifeTime>
where
	LifeTimeDefinition: From<TLifeTime>,
	TLifeTime: Clone,
{
	fn from(value: SkillBehaviorConfigData<TLifeTime>) -> Self {
		let shape = BuildSkillShape::from(value.shape);
		let contact = value.contact.into_iter().map(SkillBehavior::from);
		let projection = value.projection.into_iter().map(SkillBehavior::from);
		Self::from_shape(shape)
			.with_contact_behaviors(contact.collect())
			.with_projection_behaviors(projection.collect())
	}
}
