use bevy::prelude::*;
use common::components::asset_model::AssetModel;

const CORRIDOR_PATH_PREFIX: &str = "models/corridor/";

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("floor_forward"))]
pub(crate) struct CorridorFloorForward;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("floor_left"))]
pub(crate) struct CorridorFloorLeft;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("floor_corner_outside"))]
pub(crate) struct CorridorFloorCornerOutside;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("floor_corner_inside"))]
pub(crate) struct CorridorFloorCornerInside;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("floor"))]
pub(crate) struct CorridorFloor;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("wall_forward"))]
pub(crate) struct CorridorWallForward;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("wall_left"))]
pub(crate) struct CorridorWallLeft;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("wall_corner_outside"))]
pub(crate) struct CorridorWallCornerOutside;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("wall_corner_outside_diagonal"))]
pub(crate) struct CorridorWallCornerOutsideDiagonal;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("wall_corner_inside"))]
pub(crate) struct CorridorWallCornerInside;

#[derive(Component, Debug, PartialEq)]
#[require(AssetModel = corridor("wall"))]
pub(crate) struct CorridorWall;

fn corridor(suffix: &'static str) -> AssetModel {
	AssetModel::path(format!("{CORRIDOR_PATH_PREFIX}{suffix}.glb#Scene0"))
}
