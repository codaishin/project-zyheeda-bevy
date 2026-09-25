use crate::components::{
	camera_labels::AgentsPass,
	los::{LoS, LoSCameras, LoSCamerasHeight},
	model_render_layers::ModelRenderLayers,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub(crate) struct RoleAssigned;

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(RoleAssigned, ModelRenderLayers::from(AgentsPass), LoSCamerasHeight)]
pub(crate) struct Player;

impl Prefab<()> for Player {
	type TError = Unreachable;
	type TSystemParam = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		entity.try_insert(related!(LoSCameras[
			LoS::Right,
			LoS::Left,
			LoS::Up,
			LoS::Down,
			LoS::Forward,
			LoS::Backward,
		]));

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(RoleAssigned, ModelRenderLayers::from(AgentsPass))]
pub(crate) struct Enemy;
