use crate::components::label::UILabel;
use bevy::prelude::*;
use common::traits::handles_localization::Token;
use std::path::PathBuf;

pub(crate) type IconFallbackLabel = UILabel<Token>;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(IconFallbackLabel = UILabel(Token::from("no-icon-image")))]
pub(crate) enum Icon {
	ImagePath(PathBuf),
	Loading(Handle<Image>),
	Loaded(Handle<Image>),
	None,
}

impl Icon {
	pub(crate) fn has_image(&self) -> bool {
		self != &Self::None
	}
}
