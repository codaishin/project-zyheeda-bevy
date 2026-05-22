use bevy::prelude::*;
use common::{traits::handles_map_generation::PrefabType, zyheeda_commands::ZyheedaEntityCommands};

#[derive(Resource, Debug)]
pub struct PrefabRegister<T>(pub(crate) fn(ZyheedaEntityCommands, T::TTranslation, T))
where
	T: PrefabType;

impl<T> PrefabRegister<T>
where
	T: PrefabType,
{
	fn noop(_: ZyheedaEntityCommands, _: T::TTranslation, _: T) {}

	pub(crate) fn apply(&self, entity: ZyheedaEntityCommands, translation: T::TTranslation, t: T) {
		(self.0)(entity, translation, t)
	}
}

impl<T> Default for PrefabRegister<T>
where
	T: PrefabType,
{
	fn default() -> Self {
		Self(Self::noop)
	}
}

impl<T> PartialEq for PrefabRegister<T>
where
	T: PrefabType,
{
	fn eq(&self, other: &Self) -> bool {
		std::ptr::fn_addr_eq(self.0, other.0)
	}
}
