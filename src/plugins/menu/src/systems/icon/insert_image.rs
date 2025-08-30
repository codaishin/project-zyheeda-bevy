use crate::{Icon, components::icon::IconImage};
use bevy::prelude::*;

impl Icon {
	pub(crate) fn insert_image(
		mut commands: Commands,
		icons: Query<(Entity, &Icon), Changed<Icon>>,
	) {
		for (entity, icon) in &icons {
			let IconImage::Loaded(handle) = &icon.image else {
				continue;
			};
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.try_insert(ImageNode::new(handle.clone()));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::IconImage;
	use common::traits::handles_localization::localized::Localized;
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Icon::insert_image);

		app
	}

	#[test]
	fn insert_image() {
		let handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from(String::default()),
				image: IconImage::Loaded(handle.clone()),
			})
			.id();

		app.update();

		assert_eq!(
			Some(&handle),
			app.world()
				.entity(entity)
				.get::<ImageNode>()
				.map(|node| &node.image)
		);
	}

	#[test]
	fn do_not_insert_image_twice() {
		let handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from(String::default()),
				image: IconImage::Loaded(handle),
			})
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<ImageNode>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<ImageNode>()
				.map(|node| &node.image)
		);
	}

	#[test]
	fn insert_image_again_if_icon_changed() {
		let handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from(String::default()),
				image: IconImage::Loaded(handle.clone()),
			})
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<ImageNode>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Icon>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&handle),
			app.world()
				.entity(entity)
				.get::<ImageNode>()
				.map(|node| &node.image)
		);
	}
}
