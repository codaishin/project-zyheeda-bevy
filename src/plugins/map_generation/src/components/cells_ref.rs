use bevy::prelude::*;
use std::marker::PhantomData;

/// Used to refer from child grids to a parent with
/// the information about the grid cells
#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct CellsRef<T> {
	pub(crate) cell_definition: Entity,
	_p: PhantomData<T>,
}

impl<T> CellsRef<T> {
	pub(crate) fn from_grid_definition(entity: Entity) -> Self {
		Self {
			cell_definition: entity,
			_p: PhantomData,
		}
	}
}
