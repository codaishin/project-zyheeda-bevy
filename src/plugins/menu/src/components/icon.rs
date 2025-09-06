use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Component, Debug, PartialEq, Clone, Default)]
pub(crate) enum Icon {
	#[default]
	None,
	ImagePath(PathBuf),
	Load(Handle<Image>),
	Loaded(Handle<Image>),
}

impl Icon {
	pub(crate) fn has_image(&self) -> bool {
		self != &Self::None
	}
}
