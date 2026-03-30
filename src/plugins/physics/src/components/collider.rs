use crate::components::async_collider::{AsyncCollider, ColliderType, Source};
use bevy::{
	ecs::{entity::EntityHashSet, system::StaticSystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::{
	errors::Unreachable,
	tools::Units,
	traits::{
		handles_physics::physical_bodies::Shape,
		prefab::{Prefab, PrefabEntityCommands, Reapply},
	},
};
use std::marker::PhantomData;

pub(crate) const GENERIC_COLLISION_GROUP: Group = Group::GROUP_1;
pub(crate) const TERRAIN_GROUP: Group = Group::GROUP_2;
pub(crate) const RAY_GROUP: Group = Group::GROUP_3;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[component(immutable)]
#[require(Transform)]
pub(crate) enum ColliderShape {
	Sphere {
		radius: Units,
		hollow: bool,
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
	CustomAsset {
		mesh: &'static str,
		scale: ColliderScale,
		collider_type: ColliderType,
	},
	EntityMesh {
		collider_type: ColliderType,
	},
}

impl From<Shape> for ColliderShape {
	fn from(value: Shape) -> Self {
		match value {
			Shape::Sphere { radius } => Self::Sphere {
				radius,
				hollow: false,
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
			Shape::StaticGltfMesh3d => Self::EntityMesh {
				collider_type: ColliderType::Concave,
			},
		}
	}
}

impl Prefab<()> for ColliderShape {
	type TError = Unreachable;
	type TSystemParam<'w, 's> = ();

	const REAPPLY: Reapply = Reapply::Always;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<()>,
	) -> Result<(), Self::TError> {
		entity.try_remove::<(Collider, AsyncCollider)>();
		entity.try_insert_if_new((
			CollidingEntities::default(),
			ActiveEvents::COLLISION_EVENTS,
			ActiveCollisionTypes::default(),
		));

		let collider = match *self {
			Self::Sphere {
				radius,
				hollow: false,
			} => SyncOrAsync::Sync(Collider::ball(*radius)),
			Self::Sphere {
				radius,
				hollow: true,
			} => SyncOrAsync::Async(AsyncCollider {
				source: Source::Path("models/icosphere.glb#Mesh0/Primitive0"),
				scale: Some(ColliderScale::Absolute(Vec3::splat(*radius * 2.))),
				collider_type: ColliderType::Concave,
			}),
			Self::Cuboid {
				half_x,
				half_y,
				half_z,
			} => SyncOrAsync::Sync(Collider::cuboid(*half_x, *half_y, *half_z)),
			Self::Cylinder { half_y, radius } => {
				SyncOrAsync::Sync(Collider::cylinder(*half_y, *radius))
			}
			Self::Capsule { half_y, radius } => {
				SyncOrAsync::Sync(Collider::capsule_y(*half_y, *radius))
			}
			Self::CustomAsset {
				scale,
				mesh,
				collider_type,
			} => SyncOrAsync::Async(AsyncCollider {
				source: Source::Path(mesh),
				scale: Some(scale),
				collider_type,
			}),
			Self::EntityMesh { collider_type } => SyncOrAsync::Async(AsyncCollider {
				source: Source::MeshOfEntity,
				scale: None,
				collider_type,
			}),
		};

		match collider {
			SyncOrAsync::Sync(collider) => entity.try_insert(collider),
			SyncOrAsync::Async(collider) => entity.try_insert(collider),
		};

		Ok(())
	}
}

enum SyncOrAsync {
	Sync(Collider),
	Async(AsyncCollider),
}

/// Links a [`Collider`] entity to the corresponding `T` entity.
#[derive(Component, PartialEq, Debug)]
#[relationship(relationship_target = Colliders<T>)]
#[require(Collider, Transform, ActiveEvents, ActiveCollisionTypes)]
pub(crate) struct ChildCollider<T>
where
	T: Component,
{
	#[relationship]
	pub(crate) root: Entity,
	_p: PhantomData<T>,
}

impl<T> ChildCollider<T>
where
	T: Component,
{
	pub(crate) const fn of(entity: Entity) -> Self {
		Self {
			root: entity,
			_p: PhantomData,
		}
	}
}

#[derive(Component, PartialEq, Debug)]
#[relationship_target(relationship = ChildCollider<T>)]
pub(crate) struct Colliders<T>
where
	T: Component,
{
	#[relationship]
	entities: EntityHashSet,
	_p: PhantomData<T>,
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy_rapier3d::prelude::{ActiveCollisionTypes, ActiveEvents, CollidingEntities};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(
			|on_insert: On<Insert, ColliderShape>,
			 mut commands: Commands,
			 shapes: Query<&ColliderShape>,
			 param: StaticSystemParam<()>| {
				let shape = shapes.get(on_insert.entity).unwrap();
				let mut entity = commands.get_entity(on_insert.entity).unwrap();
				let Ok(()) = shape.insert_prefab_components(&mut entity, param);
			},
		);

		app
	}

	#[test_case(ColliderShape::Sphere {radius: Units::from(1.), hollow: false}; "sphere")]
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

	const SYNC: ColliderShape = ColliderShape::Sphere {
		radius: Units::from_u8(1),
		hollow: false,
	};
	const ASYNC: ColliderShape = ColliderShape::Sphere {
		radius: Units::from_u8(1),
		hollow: true,
	};

	fn colliders_count(entity: EntityWorldMut) -> usize {
		let mut count = 0;

		if entity.contains::<AsyncCollider>() {
			count += 1;
		}

		if entity.contains::<Collider>() {
			count += 1;
		}

		count
	}

	#[test_case([SYNC, ASYNC]; "sync async")]
	#[test_case([ASYNC, SYNC]; "async sync")]
	fn remove_previous_variations(shapes: [ColliderShape; 2]) {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(shapes[0]);
		entity.insert(shapes[1]);

		assert_eq!(1, colliders_count(entity));
	}
}
