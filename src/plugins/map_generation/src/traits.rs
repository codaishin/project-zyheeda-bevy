pub(crate) mod app;
pub(crate) mod corner;
pub(crate) mod light;
pub(crate) mod light_cell;
pub(crate) mod map;
pub(crate) mod map_cell;
pub(crate) mod wall;

use bevy::{
	app::App,
	ecs::{schedule::ScheduleLabel, system::Commands},
	reflect::TypePath,
	transform::components::Transform,
};
use bevy_rapier3d::geometry::Collider;
use common::traits::load_asset::Path;

use self::map::Cross;

pub(crate) struct CellIsEmpty;

pub(crate) trait ColliderDefinition {
	const IS_TARGET: bool;
	fn target_names() -> Vec<String>;
	fn collider() -> Collider;
}

pub(crate) trait CellDistance {
	const CELL_DISTANCE: f32;
}

pub(crate) trait Spawn {
	fn spawn(&self, commands: &mut Commands, at: Transform);
}

pub trait SourcePath {
	fn source_path() -> Path;
}

pub trait RegisterMapCell {
	fn register_map_cell<TCell: TypePath + Send + Sync + From<Cross> + SourcePath>(
		&mut self,
		label: impl ScheduleLabel,
	) -> &mut App;
}
