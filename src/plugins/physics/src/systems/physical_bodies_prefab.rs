use crate::components::{
	blocker_types::BlockerTypes,
	collider::ColliderShape,
	no_hover::NoMouseHover,
	physical_body::PhysicalBody,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::physical_bodies::PhysicsType},
	zyheeda_commands::ZyheedaCommands,
};

impl PhysicalBody {
	pub(crate) fn prefab(
		mut commands: ZyheedaCommands,
		bodies: Query<(Entity, &Self), Changed<Self>>,
	) {
		for (entity, PhysicalBody(body)) in &bodies {
			commands.try_apply_on(&entity, |mut e| {
				match body.physics_type {
					PhysicsType::Agent => {
						e.try_insert((RigidBody::KinematicPositionBased, Self::agent_controller()));
					}
					PhysicsType::Terrain => {
						e.try_insert(RigidBody::Fixed);
					}
				};

				e.try_insert((
					BlockerTypes(body.blocker_types.clone()),
					ColliderShape::from(body.shape),
				));

				if body.physics_type != PhysicsType::Terrain {
					return;
				}

				e.try_insert(NoMouseHover);
			});
		}
	}

	fn agent_controller() -> KinematicCharacterController {
		KinematicCharacterController::default()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::Units,
		traits::handles_physics::physical_bodies::{Blocker, Body, Shape},
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, PhysicalBody::prefab);

		app
	}

	#[test]
	fn insert_collider() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(shape)))
			.id();

		app.update();

		assert_eq!(
			Some(&ColliderShape::from(shape)),
			app.world().entity(entity).get::<ColliderShape>(),
		);
	}

	#[test_case(PhysicsType::Terrain, RigidBody::Fixed, false;  "terrain")]
	#[test_case(PhysicsType::Agent, RigidBody::KinematicPositionBased, true; "agent")]
	fn insert_physics_constraints(
		physics_type: PhysicsType,
		rigid_body: RigidBody,
		has_character_controller: bool,
	) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(physics_type),
			))
			.id();

		app.update();

		assert_eq!(
			(Some(&rigid_body), has_character_controller),
			(
				app.world().entity(entity).get::<RigidBody>(),
				app.world()
					.entity(entity)
					.contains::<KinematicCharacterController>(),
			)
		);
	}

	#[test]
	fn insert_blocker_types() {
		let mut app = setup();
		let blocks = [Blocker::Force, Blocker::Physical];
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_blocker_types(blocks),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&BlockerTypes::from(blocks)),
			app.world().entity(entity).get::<BlockerTypes>(),
		);
	}

	#[test_case(PhysicsType::Terrain, Some(&NoMouseHover); "no mouse hover on terrain")]
	#[test_case(PhysicsType::Agent, None; "mouse hover on agent")]
	fn insert_mouse_hover_control(
		collider_type: PhysicsType,
		no_mouse_hover: Option<&NoMouseHover>,
	) {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(
				Body::from_shape(shape).with_physics_type(collider_type),
			))
			.id();

		app.update();

		assert_eq!(
			no_mouse_hover,
			app.world().entity(entity).get::<NoMouseHover>(),
		);
	}

	#[test]
	fn do_nothing_when_not_changed() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(Shape::Sphere {
				radius: Units::from(11.),
			})))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(ColliderShape::from(shape));
		app.update();

		assert_eq!(
			Some(&ColliderShape::from(shape)),
			app.world().entity(entity).get::<ColliderShape>()
		);
	}

	#[test]
	fn reinsert_collider() {
		let mut app = setup();
		let shape = Shape::Sphere {
			radius: Units::from(42.),
		};
		let entity = app
			.world_mut()
			.spawn(PhysicalBody(Body::from_shape(Shape::Sphere {
				radius: Units::from(11.),
			})))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(PhysicalBody(Body::from(shape)));
		app.update();

		assert_eq!(
			Some(&ColliderShape::from(shape)),
			app.world().entity(entity).get::<ColliderShape>(),
		);
	}
}
