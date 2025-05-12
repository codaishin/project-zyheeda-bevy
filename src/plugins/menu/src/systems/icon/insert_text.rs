use crate::{Icon, components::icon::IconImage};
use bevy::prelude::*;

impl Icon {
	pub(crate) fn insert_text(
		mut commands: Commands,
		icons: Query<(Entity, &Icon), Changed<Icon>>,
	) {
		for (entity, icon) in &icons {
			if icon.image != IconImage::None {
				continue;
			}
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.try_insert(Text::from(icon.localized.clone()));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::icon::IconImage;
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::handles_localization::localized::Localized,
	};
	use std::path::PathBuf;
	use test_case::test_case;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Icon::insert_text);

		app
	}

	#[test]
	fn insert_text_when_no_icon_image() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my description"),
				image: IconImage::None,
			})
			.id();

		app.update();

		assert_eq!(
			Some("my description"),
			app.world()
				.entity(entity)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		);
	}

	#[test_case(IconImage::Path(PathBuf::from("")); "is path")]
	#[test_case(IconImage::Loading(new_handle()); "is loading")]
	#[test_case(IconImage::Loaded(new_handle()); "is loaded")]
	fn do_not_insert_text_when_image(image: IconImage) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my description"),
				image,
			})
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		);
	}

	#[test]
	fn do_not_insert_text_twice() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my description"),
				image: IconImage::None,
			})
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Text>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		);
	}

	#[test]
	fn insert_text_again_when_icon_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my description"),
				image: IconImage::None,
			})
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Text>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Icon>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some("my description"),
			app.world()
				.entity(entity)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		);
	}
}
