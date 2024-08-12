pub(crate) mod colors;
pub(crate) mod combo_tree_layout;
pub(crate) mod get_bundle;
pub(crate) mod get_node;
pub(crate) mod instantiate_content_on;
pub(crate) mod set;
pub(crate) mod tooltip_ui_control;

use crate::tools::Layout;
use bevy::{ecs::system::EntityCommands, prelude::Bundle, ui::Style};
use combo_tree_layout::ComboTreeLayout;
use common::traits::load_asset::LoadAsset;
use get_node::GetNode;
use instantiate_content_on::InstantiateContentOn;

pub(crate) trait UpdateCombosView {
	fn update_combos_view(&mut self, combos: ComboTreeLayout);
}

pub(crate) trait UI: GetNode + InstantiateContentOn {}

impl<T: GetNode + InstantiateContentOn> UI for T {}

pub(crate) trait LoadUi<TAssetServer: LoadAsset> {
	fn load_ui(server: &mut TAssetServer) -> Self;
}

pub(crate) trait RootStyle {
	fn root_style(&self) -> Style;
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

pub(crate) trait GetBundle
where
	Self::TBundle: Bundle,
{
	type TBundle;
	fn bundle(&self) -> Option<Self::TBundle>;
}
