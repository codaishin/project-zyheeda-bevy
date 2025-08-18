use crate::tools::action_key::slot::SlotKey;
use bevy::{asset::AssetPath, ecs::component::Component};

pub trait LoadoutConfig: Component {
	fn inventory(&self) -> impl IntoIterator<Item = Option<AssetPath<'static>>>;
	fn slots(&self) -> impl IntoIterator<Item = (SlotKey, Option<AssetPath<'static>>)>;
}
