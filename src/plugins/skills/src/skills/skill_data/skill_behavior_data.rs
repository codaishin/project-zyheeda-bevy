pub(crate) mod shape_data;
pub(crate) mod start_behavior_data;

use crate::{
	behaviors::{
		build_skill_shape::BuildSkillShape,
		start_behavior::SkillBehavior,
		SkillBehaviorConfig,
	},
	skills::RunSkillBehavior,
	traits::skill_builder::LifeTimeDefinition,
};
use serde::{Deserialize, Serialize};
use shape_data::SkillShapeData;
use start_behavior_data::SkillBehaviorData;
use std::time::Duration;

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OnActiveLifetime {
	UntilOutlived(Duration),
	Infinite,
}

impl From<OnActiveLifetime> for LifeTimeDefinition {
	fn from(value: OnActiveLifetime) -> Self {
		match value {
			OnActiveLifetime::UntilOutlived(duration) => Self::UntilOutlived(duration),
			OnActiveLifetime::Infinite => Self::Infinite,
		}
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum OnAimLifeTime {
	UntilStopped,
	Infinite,
}

impl From<OnAimLifeTime> for LifeTimeDefinition {
	fn from(value: OnAimLifeTime) -> Self {
		match value {
			OnAimLifeTime::UntilStopped => Self::UntilStopped,
			OnAimLifeTime::Infinite => Self::Infinite,
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum RunSkillBehaviorData {
	OnActive(SkillBehaviorConfigData<OnActiveLifetime>),
	OnAim(SkillBehaviorConfigData<OnAimLifeTime>),
}

impl From<RunSkillBehaviorData> for RunSkillBehavior {
	fn from(value: RunSkillBehaviorData) -> Self {
		match value {
			RunSkillBehaviorData::OnActive(v) => Self::OnActive(SkillBehaviorConfig::from(v)),
			RunSkillBehaviorData::OnAim(v) => Self::OnAim(SkillBehaviorConfig::from(v)),
		}
	}
}
