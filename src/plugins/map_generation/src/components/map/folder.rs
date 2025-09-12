use crate::components::map::cells::agent::Agent;
use bevy::prelude::*;
use common::traits::{
	register_derived_component::{DerivableFrom, InsertDerivedComponent},
	thread_safe::ThreadSafe,
};
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, path::PathBuf};

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub(crate) struct MapFolder<TCell> {
	pub(crate) path: PathBuf,
	#[serde(skip)]
	_c: PhantomData<TCell>,
}

impl<TCell, TPath> From<TPath> for MapFolder<TCell>
where
	TPath: Into<PathBuf>,
{
	fn from(path: TPath) -> Self {
		Self {
			path: path.into(),
			_c: PhantomData,
		}
	}
}

impl<'w, 's, TCell> DerivableFrom<'w, 's, MapFolder<TCell>> for MapFolder<Agent<TCell>>
where
	TCell: ThreadSafe,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;

	type TParam = ();

	fn derive_from(_: Entity, MapFolder { path, .. }: &MapFolder<TCell>, _: &()) -> Option<Self> {
		Some(Self::from(path.clone()))
	}
}
