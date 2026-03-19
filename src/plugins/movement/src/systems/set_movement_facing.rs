use crate::components::{facing::SetFace, ongoing_movement::OngoingMovement};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_movement::MovementTarget,
		handles_orientation::Face,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl OngoingMovement {
	pub(crate) fn set_facing(
		mut commands: ZyheedaCommands,
		mut removed: RemovedComponents<Self>,
		changed: Query<(Entity, &Self), Changed<Self>>,
	) {
		for entity in removed.read() {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SetFace>();
			});
		}

		for (entity, movement) in &changed {
			commands.try_apply_on(&entity, |mut e| {
				match &movement {
					OngoingMovement::Target(MovementTarget::Point(pos)) => {
						e.try_insert(SetFace(Face::Translation(*pos)));
					}
					OngoingMovement::Target(MovementTarget::Dir(dir3)) => {
						e.try_insert(SetFace(Face::Direction(*dir3)));
					}
					OngoingMovement::Stopped => {
						e.try_remove::<SetFace>();
					}
				};
			});
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, OngoingMovement::set_facing);

		app
	}

	#[test]
	fn set_to_face_translation_on_update() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(OngoingMovement::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::new(1., 2., 3.)))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	#[test]
	fn do_not_set_to_face_translation_on_update_when_not_added() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(OngoingMovement::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<SetFace>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn set_to_face_translation_on_update_when_added() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(OngoingMovement::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(OngoingMovement::target(Vec3::new(3., 4., 5.)));
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::new(3., 4., 5.)))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	#[test]
	fn set_to_face_direction_on_update_when_added() {
		let mut app = setup();
		let entity = app.world_mut().spawn(OngoingMovement::target(Dir3::NEG_X)).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(OngoingMovement::target(Dir3::NEG_Z));
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Direction(Dir3::NEG_Z))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	#[test]
	fn remove_set_face_on_update_when_stopped() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((OngoingMovement::Stopped, SetFace(Face::Target)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<OngoingMovement>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn remove_set_face_on_update_when_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((OngoingMovement::target(Dir3::NEG_X), SetFace(Face::Target)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<OngoingMovement>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn when_movement_inserted_after_removal_in_same_frame_add_face() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((OngoingMovement::target(Dir3::NEG_X), SetFace(Face::Target)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<OngoingMovement>()
			.insert(OngoingMovement::target(Dir3::NEG_X));
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Direction(Dir3::NEG_X))),
			app.world().entity(entity).get::<SetFace>()
		);
	}
}
