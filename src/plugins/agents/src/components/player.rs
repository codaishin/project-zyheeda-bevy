use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::{ErrorData, Level},
	traits::{
		accessors::get::TryGetContextMut,
		handles_animations::AnimationPriority,
		handles_light::{Light, Lumen, SetLight, TorchLight},
		handles_map_generation::AgentType,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::fmt::Display;

#[derive(Component, Default, Debug, PartialEq, Clone)]
#[component(immutable)]
#[require(Name = "Player")]
pub struct Player;

impl From<Player> for AgentType {
	fn from(_: Player) -> Self {
		Self::Player
	}
}

impl<TLights> Prefab<TLights> for Player
where
	TLights: for<'c> TryGetContextMut<TorchLight, TContext<'c>: SetLight>,
{
	type TError = CannotInsertTorchLight;
	type TSystemParam<'w, 's> = TLights;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		mut lights: StaticSystemParam<Self::TSystemParam<'_, '_>>,
	) -> Result<(), Self::TError> {
		let entity = entity.entity_id();

		let Some(mut ctx) = TLights::try_get_context_mut(&mut lights, TorchLight { entity }) else {
			return Err(CannotInsertTorchLight { entity });
		};

		ctx.set_light(Light {
			intensity: Lumen::from(10_000),
		});

		Ok(())
	}
}

pub struct CannotInsertTorchLight {
	entity: Entity,
}

impl Display for CannotInsertTorchLight {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}: cannot insert torch light", self.entity)
	}
}

impl ErrorData for CannotInsertTorchLight {
	fn level(&self) -> common::errors::Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Player lighting error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

struct Idle;

impl From<Idle> for AnimationPriority {
	fn from(_: Idle) -> Self {
		AnimationPriority::Low
	}
}
