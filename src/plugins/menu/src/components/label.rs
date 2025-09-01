use bevy::prelude::*;
use common::traits::handles_localization::localized::Localized;

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub(crate) struct UILabel<TValue = Localized>(pub(crate) TValue);
