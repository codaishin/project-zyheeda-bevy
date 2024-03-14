use bevy::{asset::Asset, math::primitives::Direction3d, reflect::TypePath};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Shape {
	Single,
	End,
	Straight,
	Cross2,
	Cross3,
	Cross4,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Cell {
	Corridor(Direction3d, Shape),
	Empty,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Cells(pub Vec<Vec<Cell>>);

#[derive(TypePath, Asset, Debug, PartialEq)]
pub struct Map(pub Cells);
