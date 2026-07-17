use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::{ErrorData, Level},
	traits::{
		accessors::get::TryGetContextMut,
		handles_map_generation::{InteractiveType, MapPrefabs, SetPrefab},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::components::{container::Container, door::Door};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[savable_component(id = "interactive")]
pub(crate) struct Interactive {
	pub(crate) interactive_type: InteractiveType,
}

impl Interactive {
	fn map_prefab(
		mut entity: ZyheedaEntityCommands,
		translation: Vec3,
		interactive_type: InteractiveType,
	) {
		entity.try_insert(Transform::from_translation(translation));

		match interactive_type {
			InteractiveType::Door => entity.try_insert(Door),
			InteractiveType::Container => entity.try_insert(Container),
		};
	}

	pub(crate) fn configure_map_prefab<TNewMapAgent>(
		mut new_agent: StaticSystemParam<TNewMapAgent>,
	) -> Result<(), NoPrefabContext>
	where
		TNewMapAgent: for<'c> TryGetContextMut<
				MapPrefabs<InteractiveType>,
				TContext<'c>: SetPrefab<InteractiveType>,
			>,
	{
		let Some(mut ctx) = TNewMapAgent::try_get_context_mut(&mut new_agent, MapPrefabs::KEY)
		else {
			return Err(NoPrefabContext);
		};

		ctx.set_prefab(Self::map_prefab);

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub struct NoPrefabContext;

impl Display for NoPrefabContext {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Cannot set interactive prefab due to missing prefab context in map plugin"
		)
	}
}

impl ErrorData for NoPrefabContext {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"No Prefab Context"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}
