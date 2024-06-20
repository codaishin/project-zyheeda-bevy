pub mod children;
pub mod colors;
pub mod get_style;
pub mod set;

use bevy::{asset::Handle, render::texture::Image};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SkillDescriptor<TKey, TIcon: Clone> {
	pub name: &'static str,
	pub key: TKey,
	pub icon: Option<TIcon>,
}

pub(crate) type CombosDescriptor<TKey, TIcon> = Vec<Vec<SkillDescriptor<TKey, TIcon>>>;

pub(crate) trait UpdateCombos<TKey> {
	fn update_combos(&mut self, combos: CombosDescriptor<TKey, Handle<Image>>);
}
