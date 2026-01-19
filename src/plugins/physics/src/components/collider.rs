use crate::{components::hollow::Hollow, physics_hooks::check_hollow_colliders::SimpleOuterRadius};
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use bevy_rapier3d::prelude::*;
use common::{
	errors::Unreachable,
	tools::Units,
	traits::{
		handles_physics::physical_bodies::Shape,
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};

#[derive(Component, Default)]
#[relationship_target(relationship = ColliderOf)]
pub(crate) struct Colliders(EntityHashSet);

#[derive(Component)]
#[relationship(relationship_target= Colliders)]
pub(crate) struct ColliderOf(pub(crate) Entity);

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Transform)]
pub(crate) enum ColliderShape {
	Sphere {
		radius: Units,
		hollow_radius: Option<Units>,
	},
	Cuboid {
		half_x: Units,
		half_y: Units,
		half_z: Units,
	},
	Capsule {
		half_y: Units,
		radius: Units,
	},
	Cylinder {
		half_y: Units,
		radius: Units,
	},
}

impl From<Shape> for ColliderShape {
	fn from(value: Shape) -> Self {
		match value {
			Shape::Sphere { radius } => Self::Sphere {
				radius,
				hollow_radius: None,
			},
			Shape::Cuboid {
				half_x,
				half_y,
				half_z,
			} => Self::Cuboid {
				half_x,
				half_y,
				half_z,
			},
			Shape::Capsule { half_y, radius } => Self::Capsule { half_y, radius },
			Shape::Cylinder { half_y, radius } => Self::Cylinder { half_y, radius },
		}
	}
}

impl SimpleOuterRadius for ColliderShape {
	fn simple_outer_radius(&self) -> Option<Units> {
		match *self {
			Self::Sphere { radius, .. } => Some(radius),
			Self::Capsule { half_y, radius } => Some(Units::from(*half_y + *radius)),
			Self::Cylinder { .. } | Self::Cuboid { .. } => None,
		}
	}
}

impl Prefab<()> for ColliderShape {
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Self::TError> {
		entity.try_insert_if_new((
			CollidingEntities::default(),
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::default(),
		));

		match *self {
			Self::Sphere {
				radius,
				hollow_radius,
			} => {
				entity.try_insert_if_new(Collider::ball(*radius));
				let Some(hollow_radius) = hollow_radius else {
					return Ok(());
				};
				entity.try_insert_if_new(Hollow {
					radius: hollow_radius,
				});
			}
			Self::Cuboid {
				half_x,
				half_y,
				half_z,
			} => {
				entity.try_insert_if_new(Collider::cuboid(*half_x, *half_y, *half_z));
			}
			Self::Cylinder { half_y, radius } => {
				entity.try_insert_if_new(Collider::cylinder(*half_y, *radius));
			}
			Self::Capsule { half_y, radius } => {
				entity.try_insert_if_new(Collider::capsule_y(*half_y, *radius));
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy_rapier3d::prelude::{ActiveCollisionTypes, ActiveEvents, CollidingEntities};
	use common::traits::load_asset::mock::MockAssetServer;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(
			|trigger: Trigger<OnInsert, ColliderShape>,
			 mut commands: Commands,
			 shapes: Query<&ColliderShape>| {
				let shape = shapes.get(trigger.target()).unwrap();
				let mut entity = commands.get_entity(trigger.target()).unwrap();
				let Ok(()) =
					shape.insert_prefab_components(&mut entity, &mut MockAssetServer::default());
			},
		);

		app
	}

	#[test_case(ColliderShape::Sphere {radius: Units::from(1.), hollow_radius: None}; "sphere")]
	#[test_case(ColliderShape::Cuboid { half_x: Units::from(1.), half_y: Units::from(1.), half_z: Units::from(1.) }; "cube")]
	#[test_case(ColliderShape::Cylinder { half_y: Units::from(1.), radius: Units::from(1.) }; "cylinder")]
	#[test_case(ColliderShape::Capsule { half_y: Units::from(1.), radius: Units::from(1.) }; "capsule")]
	fn add_required_rapier_components(shape: ColliderShape) {
		let mut app = setup();

		let entity = app.world_mut().spawn(shape);

		assert_eq!(
			(true, true, true),
			(
				entity.contains::<CollidingEntities>(),
				entity.contains::<ActiveEvents>(),
				entity.contains::<ActiveCollisionTypes>(),
			)
		)
	}
}
