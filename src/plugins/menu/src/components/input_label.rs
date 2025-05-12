use crate::traits::colors::HasPanelColors;
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
#[require(Text, TextFont(Self::font), TextColor(Self::text_color))]
pub struct InputLabel<T, TKey>
where
	T: HasPanelColors,
{
	pub key: TKey,
	phantom_data: PhantomData<T>,
}

impl<T, TKey> InputLabel<T, TKey>
where
	T: HasPanelColors,
{
	pub fn new(key: TKey) -> Self {
		Self {
			key,
			phantom_data: PhantomData,
		}
	}

	fn font() -> TextFont {
		TextFont {
			font_size: 20.0,
			..default()
		}
	}

	fn text_color() -> TextColor {
		TextColor::from(T::PANEL_COLORS.text)
	}
}
