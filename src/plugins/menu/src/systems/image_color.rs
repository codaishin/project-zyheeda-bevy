use crate::components::ImageColorCommand;
use bevy::prelude::*;
use common::traits::try_remove_from::TryRemoveFrom;

pub(crate) fn image_color(
	mut commands: Commands,
	mut images: Query<(Entity, &mut ImageNode, &ImageColorCommand)>,
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
	use testing::SingleThreadedApp;

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
				ImageNode::new(Handle::default()),
				ImageColorCommand(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(Color::srgb(0.1, 0.2, 0.3)),
			app.world()
				.entity(image)
				.get::<ImageNode>()
				.map(|i| i.color)
		)
	}

	#[test]
	fn remove_image_color_command() {
		let mut app = setup();
		let image = app
			.world_mut()
			.spawn((
				ImageNode::new(Handle::default()),
				ImageColorCommand(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(image).get::<ImageColorCommand>());
	}
}
