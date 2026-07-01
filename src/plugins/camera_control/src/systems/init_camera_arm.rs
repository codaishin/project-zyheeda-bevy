use crate::components::camera_arm::CameraArm;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::Units,
	traits::{
		accessors::get::{Get, TryApplyOn, View},
		handles_player::PlayerEntity,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl CameraArm {
	pub(crate) fn init_for<TPlayer>(
		mut commands: ZyheedaCommands,
		arms: Query<(), With<CameraArm>>,
		player: StaticSystemParam<TPlayer>,
	) where
		TPlayer: for<'w, 's> SystemParam<Item<'w, 's>: View<PlayerEntity>>,
	{
		let Some(player) = player.view() else {
			return;
		};
		let Some(entity) = commands.get(&player) else {
			return;
		};

		if arms.contains(entity) {
			return;
		}

		commands.try_apply_on(&player, |mut p| {
			p.try_insert(Self::initial_arm());
		});
	}

	fn initial_arm() -> Self {
		let direction = Dir3::try_from(Vec3::new(1., 2., 1.)).unwrap_or(Dir3::Z);

		Self {
			distance: Units::from(15.),
			sensitivity: Units::from(0.005),
			direction,
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use common::{CommonPlugin, components::persistent_entity::PersistentEntity};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	static PLAYER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	#[derive(SystemParam)]
	struct _Player;

	impl View<PlayerEntity> for _Player {
		fn view(&self) -> Option<PersistentEntity> {
			Some(*PLAYER)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.add_systems(Update, CameraArm::init_for::<_Player>);

		app
	}

	#[test]
	fn set_orbit() {
		let mut app = setup();
		let entity = app.world_mut().spawn(*PLAYER).id();

		app.update();

		assert_eq!(
			Some(&CameraArm::initial_arm()),
			app.world().entity(entity).get::<CameraArm>(),
		);
	}

	#[test]
	fn do_not_override_existing_orbit() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				*PLAYER,
				CameraArm {
					distance: Units::from(11.),
					sensitivity: Units::from(22.),
					direction: Dir3::Y,
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&CameraArm {
				distance: Units::from(11.),
				sensitivity: Units::from(22.),
				direction: Dir3::Y,
			}),
			app.world().entity(entity).get::<CameraArm>(),
		);
	}
}
