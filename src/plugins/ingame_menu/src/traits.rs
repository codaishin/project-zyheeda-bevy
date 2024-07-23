pub mod colors;
pub mod get_node;
pub mod instantiate_content_on;
pub mod set;
pub mod tooltip_ui_control;

use crate::tools::{Layout, SkillDescriptor};
use bevy::{asset::Handle, render::texture::Image, ui::Style};
use get_node::GetNode;
use instantiate_content_on::InstantiateContentOn;

pub(crate) type CombosDescriptor<TKey, TIcon> = Vec<Vec<SkillDescriptor<TKey, TIcon>>>;

pub(crate) trait UpdateCombos<TKey> {
	fn update_combos(&mut self, combos: CombosDescriptor<TKey, Handle<Image>>);
}

pub(crate) trait UI: GetNode + InstantiateContentOn {}

impl<T: GetNode + InstantiateContentOn> UI for T {}

pub(crate) trait RootStyle {
	fn root_style(&self) -> Style;
}

pub(crate) trait GetLayout {
	fn layout(&self) -> Layout;
}
