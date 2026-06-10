use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::Unreachable,
	traits::{
		accessors::get::TryGetContextMut,
		handles_graphics::{HasNoRole, Role, SetRole},
		handles_map_generation::AgentType,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Default, Debug, PartialEq, Clone)]
#[component(immutable)]
#[require(Name = "Player")]
pub struct Player;

impl From<Player> for AgentType {
	fn from(_: Player) -> Self {
		Self::Player
	}
}

impl<TGraphics> Prefab<TGraphics> for Player
where
	TGraphics: for<'c> TryGetContextMut<HasNoRole, TContext<'c>: SetRole>,
{
	type TError = Unreachable;
	type TSystemParam<'w, 's> = TGraphics;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		mut graphics: StaticSystemParam<Self::TSystemParam<'_, '_>>,
	) -> Result<(), Self::TError> {
		let entity = entity.entity_id();
		let no_role = HasNoRole { entity };
		let Some(mut ctx) = TGraphics::try_get_context_mut(&mut graphics, no_role) else {
			return Ok(());
		};

		ctx.set_role(Role::Player);

		Ok(())
	}
}
