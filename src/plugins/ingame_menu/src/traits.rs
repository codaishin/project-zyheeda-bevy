pub mod children;
pub mod colors;
pub mod get_style;
pub mod set;

use bevy::{asset::Handle, ecs::system::EntityCommands, render::texture::Image};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SkillDescriptor<TKey: Clone, TIcon: Clone> {
	pub name: &'static str,
	pub key: TKey,
	pub icon: Option<TIcon>,
}

pub(crate) trait InsertCombo<TKey: Clone> {
	fn insert_combo<'a>(
		&'a mut self,
		entity: &'a mut EntityCommands<'a>,
		combo: Vec<SkillDescriptor<TKey, Handle<Image>>>,
	);
}
