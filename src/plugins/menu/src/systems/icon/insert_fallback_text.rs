use crate::components::{icon::Icon, label::UILabel};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl Icon {
	pub(crate) fn insert_fallback_text(
		mut commands: ZyheedaCommands,
		icons: Query<(Entity, &Icon, &UILabel), IconOrLabelChanged>,
	) {
		for (entity, icon, UILabel(label)) in &icons {
			if icon.has_image() {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Text::from(label.clone()));
			});
		}
	}
}

type IconOrLabelChanged = Or<(Changed<Icon>, Changed<UILabel>)>;

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_localization::localized::Localized;
	use std::path::PathBuf;
	use test_case::test_case;
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Icon::insert_fallback_text);

		app
	}

	#[test]
	fn insert_text_when_no_icon_image() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UILabel(Localized::from("my description")), Icon::None))
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

	#[test_case(Icon::ImagePath(PathBuf::from("")); "is path")]
	#[test_case(Icon::Loading(new_handle()); "is loading")]
	#[test_case(Icon::Loaded(new_handle()); "is loaded")]
	fn do_not_insert_text_when_icon_has_image(icon: Icon) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UILabel(Localized::from("my description")), icon))
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
			.spawn((UILabel(Localized::from("my description")), Icon::None))
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
			.spawn((UILabel(Localized::from("my description")), Icon::None))
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

	#[test]
	fn insert_text_again_when_label_added() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UILabel(Localized::from("my description")), Icon::None))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Text>();
		app.world_mut()
			.entity_mut(entity)
			.insert(UILabel(Localized::from("my description")));
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
