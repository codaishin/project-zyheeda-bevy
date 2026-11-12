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
	skill::Skill,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use bevy_rapier3d::prelude::*;
use common::prelude::*;
use std::{fmt::Display, marker::PhantomData, ops::Deref};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
#[require(Transform, CharacterGravity)]
pub(crate) struct MotionCollider {
	pub(crate) shape: Shape,
}

#[derive(Component, Debug, PartialEq, Default)]
#[require(Transform)]
pub(crate) struct OldTranslation(pub(crate) Vec3);

impl Deref for OldTranslation {
	type Target = Vec3;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = MotionControllerOf<TTarget>, linked_spawn)]
pub(crate) struct MotionControlled<TTarget>
where
	TTarget: MaybeInMotion + ThreadSafe,
{
	#[relationship]
	controller: Entity,
	_target: PhantomData<TTarget>,
}

impl<TTarget> MotionControlled<TTarget>
where
	TTarget: MaybeInMotion + ThreadSafe,
{
	pub(crate) fn controller(&self) -> Entity {
		self.controller
	}
}

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = MotionControlled<TTarget>)]
#[require(OldTranslation)]
pub(crate) struct MotionControllerOf<TTarget>
where
	TTarget: MaybeInMotion + ThreadSafe,
{
	#[relationship]
	entity: Entity,
	offset: Units,
	behavior: MotionBehavior,
	_target: PhantomData<TTarget>,
}

impl MotionControllerOf<MotionCollider> {
	pub(crate) const fn motion_collider(entity: Entity) -> Self {
		Self {
			entity,
			offset: Units::ZERO,
			behavior: MotionBehavior::Interpolated,
			_target: PhantomData,
		}
	}
}

impl MotionControllerOf<Skill> {
	pub(crate) const fn skill(entity: Entity) -> Self {
		Self {
			entity,
			offset: Units::ZERO,
			behavior: MotionBehavior::Interpolated,
			_target: PhantomData,
		}
	}
}

impl<TTarget> MotionControllerOf<TTarget>
where
	TTarget: MaybeInMotion + ThreadSafe,
{
	pub(crate) const fn with_offset(mut self, offset: Units) -> Self {
		self.offset = offset;
		self
	}

	pub(crate) fn get_offset(&self) -> Units {
		self.offset
	}

	pub(crate) fn get_entity(&self) -> Entity {
		self.entity
	}

	pub(crate) fn behavior(&self) -> MotionBehavior {
		self.behavior
	}

	pub(crate) const fn with_behavior(mut self, behavior: MotionBehavior) -> Self {
		self.behavior = behavior;
		self
	}
}

impl Prefab<()> for MotionControllerOf<MotionCollider> {
	type TError = MotionControlParametersMissing;
	type TSystemParam = Query<'static, 'static, (&'static MotionCollider, &'static Transform)>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		parameters: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), MotionControlParametersMissing> {
		let Ok((parameters, transform)) = parameters.get(self.entity) else {
			return Err(MotionControlParametersMissing(self.entity));
		};

		entity.try_insert((
			*transform,
			Physical::Contact,
			RigidBody::KinematicPositionBased,
			ColliderShape::from(parameters.shape),
			ColliderOf(self.entity),
			CollidingEntities::default(),
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::all(),
			KinematicCharacterController {
				filter_groups: Some(CollisionGroups {
					memberships: AGENTS_GROUP,
					filters: SKILLS_GROUP | TERRAIN_GROUP,
					test_mode: InteractionTestMode::And,
				}),
				offset: CharacterLength::Absolute(*self.offset),
				snap_to_ground: Some(CharacterLength::Absolute(*self.offset)),
				..default()
			},
			CollisionGroups {
				memberships: AGENTS_GROUP | MOUSE_HOVERABLE_GROUP,
				filters: AGENTS_GROUP | SKILLS_GROUP | TERRAIN_GROUP | RAY_GROUP,
				test_mode: InteractionTestMode::And,
			},
		));
		Ok(())
	}
}

pub(crate) trait MaybeInMotion: internal::MaybeInMotion {}

impl<T> MaybeInMotion for T where T: internal::MaybeInMotion {}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub(crate) enum MotionBehavior {
	#[default]
	Interpolated,
	Set,
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

mod internal {
	use super::*;

	pub(crate) trait MaybeInMotion {}

	impl MaybeInMotion for MotionCollider {}

	impl MaybeInMotion for Skill {}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		collider::{ColliderShape, MOUSE_HOVERABLE_GROUP},
		collision_domains::Physical,
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_prefab_observer::<MotionControllerOf<MotionCollider>, ()>();

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

		let entity = app
			.world_mut()
			.spawn(MotionControllerOf::motion_collider(agent));

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

		let entity = app
			.world_mut()
			.spawn(MotionControllerOf::motion_collider(agent));

		assert_eq!(Some(&ColliderOf(agent)), entity.get::<ColliderOf>());
	}

	#[test]
	fn insert_collider() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app
			.world_mut()
			.spawn(MotionControllerOf::motion_collider(agent));

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

		let entity = app
			.world_mut()
			.spawn(MotionControllerOf::motion_collider(agent));

		assert_eq!(Some(&Physical::Contact), entity.get::<Physical>());
	}

	#[test]
	fn insert_physics_constraints() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app
			.world_mut()
			.spawn(MotionControllerOf::motion_collider(agent));

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
	fn insert_controller_offset() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app
			.world_mut()
			.spawn(MotionControllerOf::motion_collider(agent).with_offset(Units::from(0.5)));

		assert_eq!(
			Some((
				CharacterLength::Absolute(0.5),
				Some(CharacterLength::Absolute(0.5))
			)),
			entity
				.get::<KinematicCharacterController>()
				.map(|c| (c.offset, c.snap_to_ground))
		);
	}

	#[test]
	fn insert_collision_groups() {
		let mut app = setup();
		let shape = Shape::Parameters(ShapeParameters::Sphere {
			radius: Units::from(42.),
		});
		let agent = app.world_mut().spawn(MotionCollider { shape }).id();

		let entity = app
			.world_mut()
			.spawn(MotionControllerOf::motion_collider(agent));

		assert_eq!(
			Some(&CollisionGroups {
				memberships: AGENTS_GROUP | MOUSE_HOVERABLE_GROUP,
				filters: AGENTS_GROUP | SKILLS_GROUP | TERRAIN_GROUP | RAY_GROUP,
				test_mode: InteractionTestMode::And,
			}),
			entity.get::<CollisionGroups>(),
		);
	}
}
