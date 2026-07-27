use crate::{components::roles::Player, resources::distance_pipeline::DistancePipelineData};
use bevy::prelude::*;

impl DistancePipelineData {
	pub(crate) fn set_player_position(
		mut data: ResMut<Self>,
		players: Query<&Transform, (With<Player>, Changed<Transform>)>,
	) {
		let Ok(Transform { translation, .. }) = players.single() else {
			return;
		};

		data.player_position = *translation;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::roles::Player;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<DistancePipelineData>();
		app.add_systems(Update, DistancePipelineData::set_player_position);

		app
	}

	#[test]
	fn set_position() {
		let mut app = setup();
		app.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)));

		app.update();

		assert_eq!(
			Vec3::new(1., 2., 3.),
			app.world()
				.resource::<DistancePipelineData>()
				.player_position
		);
	}

	#[test]
	fn ignore_no_players() {
		let mut app = setup();
		app.world_mut().spawn(Transform::from_xyz(1., 2., 3.));

		app.update();

		assert_eq!(
			Vec3::ZERO,
			app.world()
				.resource::<DistancePipelineData>()
				.player_position
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)));

		app.update();
		app.world_mut()
			.resource_mut::<DistancePipelineData>()
			.player_position = Vec3::ZERO;
		app.update();

		assert_eq!(
			Vec3::ZERO,
			app.world()
				.resource::<DistancePipelineData>()
				.player_position
		);
	}

	#[test]
	fn act_again_if_transform_changed() {
		let mut app = setup();
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
			Vec3::new(3., 2., 3.),
			app.world()
				.resource::<DistancePipelineData>()
				.player_position
		);
	}
}
