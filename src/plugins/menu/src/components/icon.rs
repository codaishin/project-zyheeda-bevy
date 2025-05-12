use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct Icon {
	pub(crate) description: String,
	pub(crate) image: IconImage,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum IconImage {
	Path(PathBuf),
	Loading(Handle<Image>),
	Loaded(Handle<Image>),
	None,
}
