use super::SourcePath;
use crate::{CurrentLevel, MapCell, map::Map, map_loader::TextLoader};
use bevy::{
	app::App,
	asset::AssetApp,
	ecs::{schedule::ScheduleLabel, system::IntoSystem},
	reflect::TypePath,
};
use common::{systems::log::log, traits::thread_safe::ThreadSafe};

pub(crate) trait LoadMapAsset {
	fn load_map_asset<TCell>(&mut self, label: impl ScheduleLabel) -> &mut App
	where
		TCell: TypePath + From<Option<char>> + SourcePath + Clone + ThreadSafe;
}

pub(crate) trait LoadMap {
	fn load_map<TCell>(&mut self, label: impl ScheduleLabel) -> &mut App;
}

impl LoadMapAsset for App {
	fn load_map_asset<TCell>(&mut self, schedule_label: impl ScheduleLabel) -> &mut App
	where
		TCell: TypePath + From<Option<char>> + SourcePath + Clone + ThreadSafe,
	{
		self.init_asset::<Map<TCell>>()
			.register_asset_loader(TextLoader::<Map<TCell>>::default())
			.add_systems(schedule_label, CurrentLevel::<TCell>::load_asset)
	}
}

impl LoadMap for App {
	fn load_map<TCell>(&mut self, label: impl ScheduleLabel) -> &mut App {
		self.add_systems(label, CurrentLevel::<MapCell>::set_graph.pipe(log))
	}
}
