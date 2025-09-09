pub(crate) mod count_down;
pub(crate) mod has_filter;

use bevy::ecs::query::{QueryData, QueryFilter, QueryItem};
use common::{
	tools::{Done, speed::Speed},
	zyheeda_commands::ZyheedaEntityCommands,
};

pub(crate) trait MovementUpdate {
	type TComponents<'a>: QueryData;
	type TConstraint: QueryFilter;

	fn update(
		&self,
		agent: &mut ZyheedaEntityCommands,
		components: QueryItem<Self::TComponents<'_>>,
		speed: Speed,
	) -> Done;
}
