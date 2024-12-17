pub(crate) mod colors;
pub(crate) mod combo_tree_layout;
pub(crate) mod get_bundle;
pub(crate) mod is_released;
pub(crate) mod tooltip_ui_control;
pub(crate) mod ui_components;
pub(crate) mod update_children;

pub mod reacts_to_menu_hotkeys;

use crate::tools::Layout;
use bevy::{ecs::system::EntityCommands, prelude::*};
use combo_tree_layout::ComboTreeLayout;
use common::traits::load_asset::LoadAsset;
use ui_components::GetUIComponents;
use update_children::UpdateChildren;

pub(crate) trait UpdateCombosView {
	fn update_combos_view(&mut self, combos: ComboTreeLayout);
}

pub(crate) trait UI: GetUIComponents + UpdateChildren {}

impl<T: GetUIComponents + UpdateChildren> UI for T {}

pub(crate) trait LoadUi<TAssetServer: LoadAsset> {
	fn load_ui(server: &mut TAssetServer) -> Self;
}

pub(crate) trait GetRootNode {
	fn root_node(&self) -> Node;
}

pub(crate) trait GetLayout {
	fn layout(&self) -> Layout;
}

pub(crate) trait InsertContentOn {
	fn insert_content_on(entity: &mut EntityCommands);
}

pub(crate) trait GetKey<TKey> {
	fn get_key<'a>(&'a self, key_path: &'a [TKey]) -> Option<&'a TKey>;
}

pub(crate) trait GetComponent
where
	Self::TComponent: Component,
{
	type TComponent;
	fn bundle(&self) -> Option<Self::TComponent>;
}
