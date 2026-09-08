use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Clone)]
#[component(immutable)]
pub(crate) enum PhysicsModel {
	Model(Model),
	Beam { half_y: Units, radius: Units },
}

impl Prefab<()> for PhysicsModel {
	type TError = Unreachable;
	type TSystemParam = ResMut<'static, Assets<Mesh>>;

	const REAPPLY: Reapply = Reapply::Always;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		mut meshes: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		match self {
			PhysicsModel::Model(model) => {
				entity.try_insert(model.clone());
			}
			PhysicsModel::Beam {
				half_y: half_length,
				radius,
			} => {
				entity.try_insert(Mesh3d(meshes.add(Mesh::from(Capsule3d {
					half_length: **half_length,
					radius: **radius,
				}))));
			}
		}
		Ok(())
	}
}
