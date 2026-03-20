use crate::components::collider::{GENERIC_COLLISION_GROUP, RAY_GROUP, TERRAIN_GROUP};
use bevy::{
	ecs::{query::QueryEntityError, system::StaticSystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::traits::{
	handles_physics::physical_bodies::{Body, PhysicsType},
	prefab::{Prefab, PrefabEntityCommands},
};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub struct PhysicalBody(pub(crate) Body);

impl From<Body> for PhysicalBody {
	fn from(body: Body) -> Self {
		Self(body)
	}
}

impl Prefab<()> for PhysicalBody {
	type TError = QueryEntityError;
	type TSystemParam<'w, 's> = Query<'w, 's, &'static PhysicalBody>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		bodies: StaticSystemParam<Self::TSystemParam<'_, '_>>,
	) -> Result<(), Self::TError> {
		let Self(Body { physics_type, .. }) = bodies.get(entity.entity_id())?;
		let groups = match physics_type {
			PhysicsType::Agent => GENERIC_COLLISION_GROUP,
			PhysicsType::Terrain => GENERIC_COLLISION_GROUP | TERRAIN_GROUP,
		};

		entity.try_insert(CollisionGroups::new(
			groups,
			GENERIC_COLLISION_GROUP | RAY_GROUP,
		));

		Ok(())
	}
}
