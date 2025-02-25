use super::iterate::Iterate;
use bevy::prelude::*;

pub trait HandlesMapGeneration
where
	for<'a> Self::TMap: Component + Iterate<TItem<'a> = &'a NavCell>,
{
	type TMap;
}

#[derive(Component, Debug, PartialEq, Default)]
pub struct NavCell {
	pub translation: Vec3,
	pub is_walkable: bool,
}
