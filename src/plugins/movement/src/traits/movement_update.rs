use bevy::ecs::query::{QueryData, QueryItem};
use common::{
	tools::{Done, speed::Speed},
	zyheeda_commands::ZyheedaEntityCommands,
};

pub(crate) trait MovementUpdate {
	type TComponents: QueryData;

	fn update(
		entity: &mut ZyheedaEntityCommands,
		components: QueryItem<Self::TComponents>,
		speed: Speed,
	) -> Done;

	fn stop(entity: &mut ZyheedaEntityCommands);
}
