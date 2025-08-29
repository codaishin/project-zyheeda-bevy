use crate::components::icon::Icon;
use bevy::prelude::*;
use common::traits::{
	handles_loadout::SkillToken,
	handles_localization::{Token, localized::Localized},
};
use std::sync::LazyLock;

/// Display the localized text:
/// - as [`Tooltip`](crate::components::tooltip::Tooltip) when [`Icon`] contains an image
/// - as [`Text`] when [`Icon`] equals [`Icon::None`]
///
/// The [`Text`] is added to a (deep) child with the [`UILabelText`] marker component or
/// directly to the label's entity, if [`UILabelText`] cannot be found.
#[derive(Component, Debug, PartialEq, Clone)]
#[component(immutable)]
#[require(Icon)]
pub(crate) struct UILabel<TValue = Localized>(pub(crate) TValue);

static EMPTY: LazyLock<UILabel> = LazyLock::new(|| UILabel(Localized::from("")));

impl UILabel {
	pub(crate) fn empty() -> Self {
		EMPTY.clone()
	}
}

impl Default for UILabel {
	fn default() -> Self {
		Self::empty()
	}
}

impl<'a> From<SkillToken<'a>> for UILabel<Token> {
	fn from(SkillToken(token): SkillToken<'a>) -> Self {
		Self(token.clone())
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct UILabelText;
