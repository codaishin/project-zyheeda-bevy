use crate::traits::InsertContentOn;
use bevy::{color::Color, ecs::system::EntityCommands, ui::BackgroundColor};

pub(crate) struct Unusable;

impl Unusable {
	const BACKGROUND_COLOR: Color = Color::srgb(0.64, 0.16, 0.02);
}

impl InsertContentOn for Unusable {
	fn insert_content_on(entity: &mut EntityCommands) {
		entity.try_insert(BackgroundColor::from(Unusable::BACKGROUND_COLOR));
	}
}
