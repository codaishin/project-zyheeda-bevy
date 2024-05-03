use bevy::{asset::Handle, ecs::schedule::States, render::texture::Image};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
	#[default]
	None,
	Inventory,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Icon(pub Handle<Image>);
