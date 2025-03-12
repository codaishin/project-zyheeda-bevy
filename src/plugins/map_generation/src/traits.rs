pub(crate) mod app;
pub(crate) mod grid_start;
pub(crate) mod is_walkable;
pub(crate) mod key_mapper;
pub(crate) mod light;
pub(crate) mod map;
pub(crate) mod to_subdivided;
pub(crate) mod wall;

use self::map::MapWindow;
use bevy::{
	app::App,
	ecs::{schedule::ScheduleLabel, system::EntityCommands},
	reflect::TypePath,
};
use common::traits::{handles_lights::HandlesLights, load_asset::Path, thread_safe::ThreadSafe};

pub(crate) trait ExtraComponentsDefinition {
	fn target_names() -> Vec<String>;
	fn insert_bundle<TLights>(entity: &mut EntityCommands)
	where
		TLights: HandlesLights + ThreadSafe;
}

pub(crate) trait GridCellDistanceDefinition {
	const CELL_DISTANCE: f32;
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
