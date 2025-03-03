use super::{RegisterMapCell, SourcePath, map::MapWindow};
use crate::{LoadLevel, map::Map, map_loader::TextLoader};
use bevy::{app::App, asset::AssetApp, ecs::schedule::ScheduleLabel, reflect::TypePath};

impl RegisterMapCell for App {
	fn register_map_cell<TCell: TypePath + Send + Sync + From<MapWindow> + SourcePath>(
		&mut self,
		begin_load: impl ScheduleLabel,
	) -> &mut App {
		self.init_asset::<Map<TCell>>()
			.register_asset_loader(TextLoader::<Map<TCell>>::default())
			.add_systems(begin_load, LoadLevel::<TCell>::start)
	}
}
