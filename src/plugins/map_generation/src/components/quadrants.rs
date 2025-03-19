use bevy::prelude::*;
use common::components::AssetModel;

const CORRIDOR_PATH_PREFIX: &str = "models/corridor_";

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel(|| asset("floor")))]
pub(crate) struct CorridorFloor;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel(|| asset("wall_forward")))]
pub(crate) struct CorridorWallForward;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel(|| asset("wall_left")))]
pub(crate) struct CorridorWallLeft;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel(|| asset("wall_corner_outside")))]
pub(crate) struct CorridorWallCornerOutside;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel(|| asset("wall_corner_inside")))]
pub(crate) struct CorridorWallCornerInside;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel(|| asset("wall")))]
pub(crate) struct CorridorWall;

fn asset(suffix: &'static str) -> AssetModel {
	AssetModel::Path(format!("{}{}.glb#Scene0", CORRIDOR_PATH_PREFIX, suffix))
}
