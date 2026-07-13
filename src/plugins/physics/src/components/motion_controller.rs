use crate::components::{
	character_gravity::CharacterGravity,
	collider::{
		AGENTS_GROUP,
		ColliderOf,
		ColliderShape,
		MOUSE_HOVERABLE_GROUP,
		RAY_GROUP,
		SKILLS_GROUP,
		TERRAIN_GROUP,
	},
	collision_domains::Physical,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	errors::{ErrorData, Level},
	traits::{
		handles_physics::physical_bodies::Shape,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::fmt::Display;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
#[require(Transform, CharacterGravity)]
pub(crate) struct MotionCollider {
	pub(crate) shape: Shape,
}

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = MotionControllerOf, linked_spawn)]
pub(crate) struct MotionController(Entity);

impl MotionController {
	pub(crate) fn get(&self) -> Entity {
		self.0
	}
}

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = MotionController)]
pub(crate) struct MotionControllerOf(pub(crate) Entity);

impl Prefab<()> for MotionControllerOf {
	type TError = MotionControlParametersMissing;
	type TSystemParam = Query<'static, 'static, (&'static MotionCollider, &'static Transform)>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		parameters: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), MotionControlParametersMissing> {
		let Ok((parameters, transform)) = parameters.get(self.0) else {
			return Err(MotionControlParametersMissing(self.0));
		};

		entity.try_insert((
			*transform,
			Physical::Contact,
			RigidBody::KinematicPositionBased,
			ColliderShape::from(parameters.shape),
			ColliderOf(self.0),
			CollidingEntities::default(),
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::all(),
			KinematicCharacterController {
				filter_groups: Some(CollisionGroups {
					memberships: AGENTS_GROUP,
					filters: SKILLS_GROUP | TERRAIN_GROUP,
				}),
				..default()
			},
			CollisionGroups {
				memberships: AGENTS_GROUP | MOUSE_HOVERABLE_GROUP,
				filters: AGENTS_GROUP | SKILLS_GROUP | TERRAIN_GROUP | RAY_GROUP,
			},
		));
		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct MotionControlParametersMissing(Entity);

impl Display for MotionControlParametersMissing {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: motion control parameter missing", self.0)
	}
}

impl ErrorData for MotionControlParametersMissing {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Motion Control Parameters Missing"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		collider::{ColliderShape, MOUSE_HOVERABLE_GROUP},
		collision_domains::Physical,
	};
	use common::{
		tools::Units,
		traits::{handles_physics::physical_bodies::ShapeParameters, prefab::AddPrefabObserver},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_prefab_observer::<MotionControllerOf, ()>();

		app
	}

	#[test]
	fn copy_transform() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app
			.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), MotionCollider { shape }))
			.id();

		let entity = app.world_mut().spawn(MotionControllerOf(agent));

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 3.)),
			entity.get::<Transform>(),
		);
	}

	#[test]
	fn insert_relation() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app.world_mut().spawn(MotionControllerOf(agent));

		assert_eq!(Some(&ColliderOf(agent)), entity.get::<ColliderOf>());
	}

	#[test]
	fn insert_collider() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app.world_mut().spawn(MotionControllerOf(agent));

		assert_eq!(
			Some(&ColliderShape::from(shape)),
			entity.get::<ColliderShape>(),
		);
	}

	#[test]
	fn insert_physical_contact() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app.world_mut().spawn(MotionControllerOf(agent));

		assert_eq!(Some(&Physical::Contact), entity.get::<Physical>());
	}

	#[test]
	fn insert_physics_constraints() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app.world_mut().spawn(MotionControllerOf(agent));

		assert_eq!(
			(
				Some(&RigidBody::KinematicPositionBased),
				true,
				true,
				Some(&ActiveEvents::COLLISION_EVENTS),
				Some(&ActiveCollisionTypes::all())
			),
			(
				entity.get::<RigidBody>(),
				entity.contains::<KinematicCharacterController>(),
				entity.contains::<CollidingEntities>(),
				entity.get::<ActiveEvents>(),
				entity.get::<ActiveCollisionTypes>(),
			)
		);
	}

	#[test]
	fn insert_collision_groups() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app.world_mut().spawn(MotionControllerOf(agent));

		assert_eq!(
			Some(&CollisionGroups {
				memberships: AGENTS_GROUP | MOUSE_HOVERABLE_GROUP,
				filters: AGENTS_GROUP | SKILLS_GROUP | TERRAIN_GROUP | RAY_GROUP
			}),
			entity.get::<CollisionGroups>(),
		);
	}
}
