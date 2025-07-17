use bevy::prelude::*;
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
