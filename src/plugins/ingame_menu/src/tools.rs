pub(crate) mod menu_state;

use bevy::{asset::Handle, render::texture::Image};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Icon(pub Handle<Image>);
