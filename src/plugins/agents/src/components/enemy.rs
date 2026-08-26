pub(crate) mod attack_config;
pub(crate) mod attack_phase;
pub(crate) mod attacking;
pub(crate) mod chasing;
pub(crate) mod void_sphere;

use crate::components::enemy::attack_config::EnemyAttackConfig;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::prelude::*;
use macros::{SavableComponent, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone)]
#[component(immutable)]
#[require(PersistentEntity, Transform, Visibility, EnemyAttackConfig)]
#[savable_component(id = "enemy")]
pub struct Enemy {
	pub(crate) aggro_range: Units,
	pub(crate) attack_range: Units,
	pub(crate) min_target_distance: Option<Units>,
}

impl<TGraphics> Prefab<TGraphics> for Enemy
where
	TGraphics: for<'c> TryGetContextMut<HasNoRole, TContext<'c>: SetRole>,
{
	type TError = Unreachable;
	type TSystemParam = TGraphics;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		mut graphics: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		let entity = entity.entity_id();
		let no_role = HasNoRole { entity };
		let Some(mut ctx) = TGraphics::try_get_context_mut(&mut graphics, no_role) else {
			return Ok(());
		};

		ctx.set_role(Role::Enemy);

		Ok(())
	}
}
