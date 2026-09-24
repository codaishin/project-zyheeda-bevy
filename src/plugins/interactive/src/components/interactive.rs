use crate::components::{container::Container, door::Door};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[component(immutable)]
#[savable_component(id = "interactive")]
pub(crate) struct Interactive {
	pub(crate) interactive_type: InteractiveType,
}

impl Interactive {
	fn map_prefab(
		mut entity: ZyheedaEntityCommands,
		translation: GlobalTransform,
		interactive_type: InteractiveType,
	) {
		entity.try_insert(Transform::from(translation));

		match interactive_type {
			InteractiveType::Door => entity.try_insert(Door),
			InteractiveType::Container => entity.try_insert(Container),
		};
	}

	pub(crate) fn configure_map_prefab<TMapGeneration>(
		mut new_agent: StaticSystemParam<TMapGeneration>,
	) where
		TMapGeneration:
			for<'c> GetContextMut<InteractivePrefabs, TContext<'c>: SetPrefab<InteractiveType>>,
	{
		TMapGeneration::get_context_mut(&mut new_agent, MapPrefabs::KEY)
			.set_prefab(Self::map_prefab);
	}
}
