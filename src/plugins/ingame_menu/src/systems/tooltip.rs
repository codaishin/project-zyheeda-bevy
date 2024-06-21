use crate::{
	components::tooltip::Tooltip,
	traits::{children::Children, colors::HasBackgroundColor, get_style::GetStyle},
};
use bevy::{
	prelude::{Component, Query},
	ui::Interaction,
};
use common::traits::mouse_position::MousePosition;

pub(crate) fn tooltip<T, TWindow>(
	windows: Query<&TWindow>,
	tooltips: Query<(&Tooltip<T>, &Interaction)>,
) where
	T: Sync + Send + 'static,
	Tooltip<T>: Children + GetStyle + HasBackgroundColor,
	TWindow: Component + MousePosition,
{
}

#[cfg(test)]
mod tests {
	use super::*;
}
