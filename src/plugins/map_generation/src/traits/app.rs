use super::{map::MapWindow, RegisterMapCell, SourcePath};
use crate::{map::Map, map_loader::TextLoader, systems::begin_level_load::begin_level_load};
use bevy::{
	app::App,
	asset::{AssetApp, AssetServer},
	ecs::schedule::ScheduleLabel,
	reflect::TypePath,
};

impl RegisterMapCell for App {
	fn register_map_cell<TCell: TypePath + Send + Sync + From<MapWindow> + SourcePath>(
		&mut self,
		begin_load: impl ScheduleLabel,
	) -> &mut App {
		self.init_asset::<Map<TCell>>()
			.register_asset_loader(TextLoader::<Map<TCell>>::default())
			.add_systems(begin_load, begin_level_load::<AssetServer, TCell>)
	}
}
