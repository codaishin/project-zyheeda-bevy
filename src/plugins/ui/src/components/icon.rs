use bevy::{
	ecs::{entity::EntityHashSet, system::StaticSystemParam},
	prelude::*,
};
use common::prelude::*;
use std::path::PathBuf;

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub(crate) enum Icon {
	#[default]
	None,
	ImagePath(PathBuf),
	Load(Handle<Image>),
	Loaded(Handle<Image>),
}

impl Icon {
	pub(crate) fn has_image(&self) -> bool {
		self != &Self::None
	}
}

#[derive(Component, Debug, PartialEq, Clone, Default)]
#[relationship_target(relationship = IconStackItem)]
pub(crate) struct IconStack(EntityHashSet);

#[derive(Component, Debug, PartialEq, Clone)]
#[relationship(relationship_target = IconStack)]
pub(crate) struct IconStackItem {
	#[relationship]
	pub(crate) of: Entity,
	pub(crate) index: usize,
}

impl IconStackItem {
	pub(crate) fn of(of: Entity) -> Self {
		Self { of, index: 0 }
	}

	pub(crate) fn with_index(mut self, index: usize) -> Self {
		self.index = index;
		self
	}
}

impl Prefab<()> for IconStackItem {
	type TError = Unreachable;
	type TSystemParam = ();

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		entity.try_insert(ChildOf(self.of));

		Ok(())
	}
}
