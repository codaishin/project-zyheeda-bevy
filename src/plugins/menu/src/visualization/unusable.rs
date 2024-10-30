use crate::{components::ImageColorCommand, traits::InsertContentOn};
use bevy::{color::Color, ecs::system::EntityCommands, ui::BackgroundColor};

pub(crate) struct Unusable;

impl Unusable {
	const BACKGROUND_COLOR: Color = Color::srgb(0.97, 0.5, 0.44);
	const IMAGE_COLOR: Color = Color::srgb(0.5, 0.27, 0.11);
}

impl InsertContentOn for Unusable {
	fn insert_content_on(entity: &mut EntityCommands) {
		entity.try_insert((
			ImageColorCommand(Self::IMAGE_COLOR),
			BackgroundColor::from(Self::BACKGROUND_COLOR),
		));
	}
}
