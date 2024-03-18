pub(crate) mod app;
pub(crate) mod corner;
pub(crate) mod light;
pub(crate) mod light_cell;
pub(crate) mod map;
pub(crate) mod map_cell;
pub(crate) mod wall;

use self::map::Cross;
use bevy::{
	app::App,
	ecs::{bundle::Bundle, schedule::ScheduleLabel, system::Commands},
	reflect::TypePath,
	transform::components::Transform,
};
use common::traits::load_asset::Path;

pub(crate) struct CellIsEmpty;

pub(crate) trait Definition<TBundle: Bundle> {
	fn target_names() -> Vec<String>;
	fn bundle() -> TBundle;
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
