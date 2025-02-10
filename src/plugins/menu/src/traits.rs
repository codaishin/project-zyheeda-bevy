pub(crate) mod build_combo_tree_layout;
pub(crate) mod colors;
pub(crate) mod insert_ui_content;
pub(crate) mod is_released;
pub(crate) mod tooltip_ui_control;

pub mod reacts_to_menu_hotkeys;

use crate::tools::Layout;
use bevy::{ecs::system::EntityCommands, prelude::*};
use build_combo_tree_layout::ComboTreeLayout;
use common::traits::load_asset::LoadAsset;

pub(crate) trait UpdateCombosView<TSkill> {
	fn update_combos_view(&mut self, combos: ComboTreeLayout<TSkill>);
}

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

pub(crate) trait GetComponent {
	type TComponent: Component;
	type TInput;

	fn component(&self, input: Self::TInput) -> Option<Self::TComponent>;
}
