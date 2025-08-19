use crate::tools::action_key::slot::SlotKey;
use bevy::asset::AssetPath;

pub trait LoadoutConfig {
	fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>>;
	fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)>;
}
