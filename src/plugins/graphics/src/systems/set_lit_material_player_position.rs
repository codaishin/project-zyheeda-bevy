use crate::{
	components::{los::LoSCamerasHeight, roles::Player},
	materials::lit_material::{LitMaterial, StandardLitMaterial},
};
use bevy::prelude::*;

type MovedPlayer = (With<Player>, Changed<GlobalTransform>);

impl LitMaterial {
	pub(crate) fn set_light_position(
		mut materials: ResMut<Assets<StandardLitMaterial>>,
		players: Query<(&GlobalTransform, &LoSCamerasHeight), MovedPlayer>,
	) {
		let Ok((transform, LoSCamerasHeight(height_offset))) = players.single() else {
			return;
		};

		for (_, materials) in materials.iter_mut() {
			materials.extension.light_position =
				transform.translation() + Vec3::Y * **height_offset;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::roles::Player, materials::lit_material::StandardLitMaterial};
	use common::prelude::*;
	use testing::SingleThreadedApp;

	fn setup<const N: usize>(materials: [StandardLitMaterial; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::<StandardLitMaterial>::default();

		for asset in materials {
			_ = assets.add(asset);
		}

		app.insert_resource(assets);
		app.add_systems(Update, LitMaterial::set_light_position);

		app
	}

	#[test]
	fn set_position() {
		let mut app = setup([StandardLitMaterial::default()]);
		app.world_mut()
			.spawn((Player, GlobalTransform::from_xyz(1., 2., 3.)));

		app.update();

		assert_eq!(
			vec![Vec3::new(1., 2., 3.)],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.light_position)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn set_position_with_offset() {
		let mut app = setup([StandardLitMaterial::default()]);
		app.world_mut().spawn((
			Player,
			LoSCamerasHeight(Units::from(10.)),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.update();

		assert_eq!(
			vec![Vec3::new(1., 12., 3.)],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.light_position)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn ignore_no_players() {
		let mut app = setup([StandardLitMaterial::default()]);
		app.world_mut().spawn(GlobalTransform::from_xyz(1., 2., 3.));

		app.update();

		assert_eq!(
			vec![Vec3::ZERO],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.light_position)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup([StandardLitMaterial::default()]);
		app.world_mut()
			.spawn((Player, GlobalTransform::from_xyz(1., 2., 3.)));

		app.update();
		for (_, m) in app
			.world_mut()
			.resource_mut::<Assets<StandardLitMaterial>>()
			.iter_mut()
		{
			m.extension.light_position = Vec3::ZERO;
		}
		app.update();

		assert_eq!(
			vec![Vec3::ZERO],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.light_position)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn act_again_if_transform_changed() {
		let mut app = setup([StandardLitMaterial::default()]);
		let entity = app
			.world_mut()
			.spawn((Player, GlobalTransform::from_xyz(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(GlobalTransform::from_xyz(3., 2., 3.));
		app.update();

		assert_eq!(
			vec![Vec3::new(3., 2., 3.)],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.light_position)
				.collect::<Vec<_>>()
		);
	}
}
