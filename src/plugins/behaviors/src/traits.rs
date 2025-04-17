pub(crate) mod change_per_frame;
pub(crate) mod has_filter;

use bevy::ecs::{
	query::{QueryData, QueryFilter, QueryItem},
	system::EntityCommands,
};
use common::tools::speed::Speed;
use std::time::Duration;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) struct IsDone(pub(crate) bool);

impl From<bool> for IsDone {
	fn from(value: bool) -> Self {
		Self(value)
	}
}

pub(crate) trait MovementUpdate {
	type TComponents<'a>: QueryData;
	type TConstraint: QueryFilter;

	fn update(
		&self,
		agent: &mut EntityCommands,
		components: QueryItem<Self::TComponents<'_>>,
		speed: Speed,
		delta: Duration,
	) -> IsDone;
}
