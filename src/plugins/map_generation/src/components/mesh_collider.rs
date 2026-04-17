use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::{ErrorData, Level},
	tools::Units,
	traits::{
		accessors::get::GetContextMut,
		handles_physics::{
			ConfigureBody,
			NoBodyConfigured,
			physical_bodies::{Blocker, Body, PhysicsType, Shape},
		},
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use std::fmt::Display;

#[derive(Component, Debug, PartialEq, Default)]
#[component(immutable)]
pub(crate) struct MeshCollider;

impl<TPhysics> Prefab<TPhysics> for MeshCollider
where
	for<'c> TPhysics: GetContextMut<NoBodyConfigured, TContext<'c>: ConfigureBody>,
{
	type TError = HasAlreadyBody;
	type TSystemParam<'w, 's> = TPhysics;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		mut physics: StaticSystemParam<Self::TSystemParam<'_, '_>>,
	) -> Result<(), Self::TError> {
		let entity = entity.entity_id();
		let key = NoBodyConfigured { entity };

		let Some(mut ctx) = TPhysics::get_context_mut(&mut physics, key) else {
			return Ok(());
		};

		ctx.configure_body(
			Body::from_shape(Shape::StaticGltfMesh3d)
				.with_physics_type(PhysicsType::Terrain)
				.with_blocker_types(Blocker::Physical),
			Units::ZERO,
		);

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub struct HasAlreadyBody {
	entity: Entity,
}

impl Display for HasAlreadyBody {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: Has already a body configured", self.entity)
	}
}

impl ErrorData for HasAlreadyBody {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Has Already Body"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
