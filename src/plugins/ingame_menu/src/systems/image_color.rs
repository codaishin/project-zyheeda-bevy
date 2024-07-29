use crate::components::ImageColorCommand;
use bevy::{
	prelude::{Commands, Entity, Query},
	ui::UiImage,
};
use common::traits::try_remove_from::TryRemoveFrom;

pub(crate) fn image_color(
	mut commands: Commands,
	mut images: Query<(Entity, &mut UiImage, &ImageColorCommand)>,
) {
	for (entity, mut image, image_command) in &mut images {
		image.color = image_command.0;
		commands.try_remove_from::<ImageColorCommand>(entity);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::ImageColorCommand;
	use bevy::{
		app::{App, Update},
		asset::Handle,
		color::Color,
		ui::UiImage,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, image_color);

		app
	}

	#[test]
	fn set_image_color_via_image_color_command() {
		let mut app = setup();
		let image = app
			.world_mut()
			.spawn((
				UiImage::from(Handle::default()),
				ImageColorCommand(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.update();

		let image = app.world().entity(image).get::<UiImage>().unwrap();

		assert_eq!(Color::srgb(0.1, 0.2, 0.3), image.color)
	}

	#[test]
	fn remove_image_color_command() {
		let mut app = setup();
		let image = app
			.world_mut()
			.spawn((
				UiImage::from(Handle::default()),
				ImageColorCommand(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.update();

		let image = app.world().entity(image);

		assert_eq!(None, image.get::<ImageColorCommand>());
	}
}
