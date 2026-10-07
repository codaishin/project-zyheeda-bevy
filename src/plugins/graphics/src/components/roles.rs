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
#[require(RoleAssigned, LoSCamerasHeight)]
pub(crate) struct Player;

impl Prefab<()> for Player {
	type TError = Unreachable;
	type TSystemParam = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		entity.try_insert((
			ModelRenderLayers::from(AgentsPass),
			related!(LoSCameras[
				LoS::Right,
				LoS::Left,
				LoS::Up,
				LoS::Down,
				LoS::Forward,
				LoS::Backward,
			]),
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::camera_labels::WorldPass;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_prefab_observer::<Player, ()>();
		app.add_prefab_observer::<Enemy, ()>();

		app
	}

	#[test]
	fn override_player_render_layers() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn((ModelRenderLayers::from(WorldPass), Player))
			.id();

		assert_eq!(
			Some(&ModelRenderLayers::from(AgentsPass)),
			app.world().entity(entity).get::<ModelRenderLayers>(),
		);
	}

	#[test]
	fn override_enemy_render_layers() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn((ModelRenderLayers::from(WorldPass), Enemy))
			.id();

		assert_eq!(
			Some(&ModelRenderLayers::from(AgentsPass)),
			app.world().entity(entity).get::<ModelRenderLayers>(),
		);
	}
}
