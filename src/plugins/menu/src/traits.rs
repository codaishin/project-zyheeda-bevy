pub(crate) mod add_dropdown;
pub(crate) mod add_tooltip;
pub(crate) mod add_ui;
pub(crate) mod build_combo_tree_layout;
pub(crate) mod colors;
pub(crate) mod insert_ui_content;
pub(crate) mod is_released;
pub(crate) mod tooltip_ui_control;
pub(crate) mod trigger_game_state;
pub(crate) mod ui_traits;
pub(crate) mod update_key_bindings;

use std::fmt::Debug;

use crate::{components::combo_overview::ComboSkill, tools::Layout};
use bevy::prelude::*;
use build_combo_tree_layout::ComboTreeLayout;
use common::{
	tools::action_key::slot::SlotKey,
	traits::load_asset::LoadAsset,
	zyheeda_commands::ZyheedaEntityCommands,
};

pub(crate) trait UpdateCombosView<TId>
where
	TId: Debug + PartialEq + Clone,
{
	fn update_combos_view(&mut self, combos: ComboTreeLayout<SlotKey, ComboSkill<TId>>);
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
	fn insert_content_on(entity: &mut ZyheedaEntityCommands);
}

pub(crate) trait GetKey<TKey> {
	fn get_key<'a>(&'a self, key_path: &'a [TKey]) -> Option<&'a TKey>;
}

pub(crate) trait GetComponent {
	type TComponent: Component;
	type TInput;

	fn component(&self, input: Self::TInput) -> Option<Self::TComponent>;
}
