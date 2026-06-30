use crate::components::{
	camera_labels::{AgentsPass, VisibilityPass},
	model_render_layers::ModelRenderLayers,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::Unreachable,
	traits::prefab::{Prefab, PrefabEntityCommands},
};

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub(crate) struct RoleAssigned;

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(RoleAssigned)]
pub(crate) struct Player;

impl Prefab<()> for Player {
	type TError = Unreachable;
	type TSystemParam = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		entity
			.try_insert(ModelRenderLayers::from(AgentsPass))
			.with_child((
				ModelRenderLayers::from(VisibilityPass),
				PointLight {
					intensity: 1e30, // Set high to guarantee fully lighting all non occluded areas
					range: 20.,
					shadows_enabled: true,
					..default()
				},
			));

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(RoleAssigned)]
pub(crate) struct Enemy;

impl Prefab<()> for Enemy {
	type TError = Unreachable;
	type TSystemParam = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		entity.try_insert(ModelRenderLayers::from(AgentsPass));

		Ok(())
	}
}
