use bevy::prelude::*;
use common::traits::load_asset::Path;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub(crate) struct MapAsset<TCell> {
	pub(crate) path: Path,
	#[serde(skip)]
	_c: PhantomData<TCell>,
}

impl<TCell, TPath> From<TPath> for MapAsset<TCell>
where
	TPath: Into<Path>,
{
	fn from(path: TPath) -> Self {
		Self {
			path: path.into(),
			_c: PhantomData,
		}
	}
}
