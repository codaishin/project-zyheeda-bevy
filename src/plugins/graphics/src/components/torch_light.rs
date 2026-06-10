use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::Unreachable,
	traits::{
		handles_light::Lumen,
		prefab::{Prefab, PrefabEntityCommands, Reapply},
	},
};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct TorchLight {
	pub(crate) intensity: Lumen,
}

impl Prefab<()> for TorchLight {
	type TError = Unreachable;
	type TSystemParam<'w, 's> = ();

	const REAPPLY: Reapply = Reapply::Always;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<Self::TSystemParam<'_, '_>>,
	) -> Result<(), Self::TError> {
		match *self.intensity {
			0. => entity.try_remove::<PointLight>(),
			_ => entity.try_insert(PointLight {
				intensity: *self.intensity,
				range: 40.,
				radius: 1.,
				..default()
			}),
		};

		Ok(())
	}
}
