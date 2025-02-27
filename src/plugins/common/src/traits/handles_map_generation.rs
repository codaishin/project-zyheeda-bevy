use super::{inspect_able::InspectAble, iterate::Iterate};
use crate::tools::grid_cell_distance::GridCellDistance;
use bevy::prelude::*;

pub trait HandlesMapGeneration
where
	for<'a> Self::TMap:
		Component + Iterate<TItem<'a> = &'a NavCell> + InspectAble<GridCellDistance>,
{
	type TMap;
}

#[derive(Component, Debug, PartialEq, Default, Clone, Copy)]
pub struct NavCell {
	pub translation: Vec3,
	pub is_walkable: bool,
}
