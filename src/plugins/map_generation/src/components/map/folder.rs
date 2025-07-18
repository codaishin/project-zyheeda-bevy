use crate::components::map::cells::agent::Agent;
use bevy::prelude::*;
use common::traits::{
	register_derived_component::{DerivableComponentFrom, InsertDerivedComponent},
	thread_safe::ThreadSafe,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, path::PathBuf};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
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

impl<TCell> From<&MapFolder<TCell>> for MapFolder<Agent<TCell>> {
	fn from(MapFolder { path, .. }: &MapFolder<TCell>) -> Self {
		Self::from(path.clone())
	}
}

impl<TCell> DerivableComponentFrom<MapFolder<TCell>> for MapFolder<Agent<TCell>>
where
	TCell: ThreadSafe,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;
}
