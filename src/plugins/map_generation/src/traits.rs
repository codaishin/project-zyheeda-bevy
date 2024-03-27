pub(crate) mod app;
pub(crate) mod corner;
pub(crate) mod light;
pub(crate) mod light_cell;
pub(crate) mod map;
pub(crate) mod map_cell;
pub(crate) mod wall;

use self::map::MapWindow;
use bevy::{
	app::App,
	ecs::{
		schedule::ScheduleLabel,
		system::{Commands, EntityCommands},
	},
	reflect::TypePath,
	transform::components::Transform,
};
use common::traits::load_asset::Path;

pub(crate) trait ExtraComponentsDefinition {
	fn target_names() -> Vec<String>;
	fn insert_bundle(entity: &mut EntityCommands);
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
	fn register_map_cell<TCell: TypePath + Send + Sync + From<MapWindow> + SourcePath>(
		&mut self,
		label: impl ScheduleLabel,
	) -> &mut App;
}
