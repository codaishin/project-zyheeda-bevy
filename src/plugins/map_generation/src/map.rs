pub(crate) mod cell;

use bevy::{asset::Asset, reflect::TypePath};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Shape {
	Single,
	End,
	Straight,
	Cross2,
	Cross3,
	Cross4,
}

#[derive(TypePath, Asset, Debug, PartialEq)]
pub(crate) struct Map<TCell: TypePath + Sync + Send>(pub Vec<Vec<TCell>>);
