use crate::components::facing::SetFace;
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{TryApplyOn, View},
		handles_orientation::Face,
		handles_physics::CharacterMotion,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> SetFaceSystem for T where T: Component + View<CharacterMotion> {}

pub(crate) trait SetFaceSystem: Component + View<CharacterMotion> + Sized {
	fn set_facing(
		mut commands: ZyheedaCommands,
		mut removed: RemovedComponents<Self>,
		changed: Query<(Entity, &Self), Changed<Self>>,
	) {
		for entity in removed.read() {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SetFace>();
			});
		}

		for (entity, motion) in &changed {
			commands.try_apply_on(&entity, |mut e| {
				match motion.view() {
					CharacterMotion::ToTarget { target, .. } => {
						e.try_insert(SetFace(Face::Translation(target)));
					}
					CharacterMotion::Direction { direction, .. } => {
						e.try_insert(SetFace(Face::Direction(direction)));
					}
					CharacterMotion::Done => {
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
	use common::tools::speed::Speed;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion(CharacterMotion);

	impl View<CharacterMotion> for _Motion {
		fn view(&self) -> CharacterMotion {
			self.0
		}
	}

	impl _Motion {
		fn target(target: Vec3) -> Self {
			Self(CharacterMotion::ToTarget {
				target,
				speed: Speed::ZERO,
			})
		}

		fn direction(direction: Dir3) -> Self {
			Self(CharacterMotion::Direction {
				direction,
				speed: Speed::ZERO,
			})
		}

		fn stop() -> Self {
			Self(CharacterMotion::Done)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Motion::set_facing);

		app
	}

	#[test]
	fn set_to_face_translation_on_update() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Motion::target(Vec3::new(1., 2., 3.)))
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
			.spawn(_Motion::target(Vec3::new(1., 2., 3.)))
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
			.spawn(_Motion::target(Vec3::new(1., 2., 3.)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Motion::target(Vec3::new(3., 4., 5.)));
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Translation(Vec3::new(3., 4., 5.)))),
			app.world().entity(entity).get::<SetFace>()
		);
	}

	#[test]
	fn set_to_face_direction_on_update_when_added() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Motion::direction(Dir3::NEG_X)).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Motion::direction(Dir3::NEG_Z));
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
			.spawn((_Motion::stop(), SetFace(Face::Target)))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn remove_set_face_on_update_when_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Motion::direction(Dir3::NEG_X), SetFace(Face::Target)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Motion>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SetFace>());
	}

	#[test]
	fn when_movement_inserted_after_removal_in_same_frame_add_face() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Motion::direction(Dir3::NEG_X), SetFace(Face::Target)))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<_Motion>()
			.insert(_Motion::direction(Dir3::NEG_X));
		app.update();

		assert_eq!(
			Some(&SetFace(Face::Direction(Dir3::NEG_X))),
			app.world().entity(entity).get::<SetFace>()
		);
	}
}
