use crate::{
	components::roles::Player,
	materials::lit_material::{LitMaterial, StandardLitMaterial},
};
use bevy::prelude::*;

impl LitMaterial {
	pub(crate) fn set_player_position(
		mut materials: ResMut<Assets<StandardLitMaterial>>,
		players: Query<&Transform, (With<Player>, Changed<Transform>)>,
	) {
		let Ok(Transform { translation, .. }) = players.single() else {
			return;
		};

		for (_, materials) in materials.iter_mut() {
			materials.extension.player_position = *translation;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::roles::Player, materials::lit_material::StandardLitMaterial};
	use testing::SingleThreadedApp;

	fn setup<const N: usize>(materials: [StandardLitMaterial; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::<StandardLitMaterial>::default();

		for asset in materials {
			_ = assets.add(asset);
		}

		app.insert_resource(assets);
		app.add_systems(Update, LitMaterial::set_player_position);

		app
	}

	#[test]
	fn set_position() {
		let mut app = setup([StandardLitMaterial::default()]);
		app.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)));

		app.update();

		assert_eq!(
			vec![Vec3::new(1., 2., 3.)],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.player_position)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn ignore_no_players() {
		let mut app = setup([StandardLitMaterial::default()]);
		app.world_mut().spawn(Transform::from_xyz(1., 2., 3.));

		app.update();

		assert_eq!(
			vec![Vec3::ZERO],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.player_position)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup([StandardLitMaterial::default()]);
		app.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)));

		app.update();
		for (_, m) in app
			.world_mut()
			.resource_mut::<Assets<StandardLitMaterial>>()
			.iter_mut()
		{
			m.extension.player_position = Vec3::ZERO;
		}
		app.update();

		assert_eq!(
			vec![Vec3::ZERO],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.player_position)
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn act_again_if_transform_changed() {
		let mut app = setup([StandardLitMaterial::default()]);
		let entity = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Transform::from_xyz(3., 2., 3.));
		app.update();

		assert_eq!(
			vec![Vec3::new(3., 2., 3.)],
			app.world()
				.resource::<Assets<StandardLitMaterial>>()
				.iter()
				.map(|(_, m)| m.extension.player_position)
				.collect::<Vec<_>>()
		);
	}
}
