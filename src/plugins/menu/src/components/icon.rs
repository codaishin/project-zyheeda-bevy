use bevy::prelude::*;
use common::traits::handles_localization::localized::Localized;
use std::path::PathBuf;

#[derive(Component, Debug, PartialEq, Clone)]
pub(crate) struct Icon {
	pub(crate) localized: Localized,
	pub(crate) image: IconImage,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum IconImage {
	Path(PathBuf),
	Loading(Handle<Image>),
	Loaded(Handle<Image>),
	None,
}
