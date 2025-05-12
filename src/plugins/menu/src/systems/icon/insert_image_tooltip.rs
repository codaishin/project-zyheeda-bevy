use crate::{Icon, Tooltip, components::icon::IconImage};
use bevy::prelude::*;

impl Icon {
	pub(crate) fn insert_image_tooltip(
		mut commands: Commands,
		icons: Query<(Entity, &Icon), Changed<Icon>>,
	) {
		for (entity, icon) in &icons {
			if !matches!(icon.image, IconImage::Loaded(_)) {
				continue;
			}

			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.try_insert((Interaction::default(), Tooltip::new(icon.localized.clone())));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{Tooltip, components::icon::IconImage};
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::handles_localization::localized::Localized,
	};
	use std::path::PathBuf;
	use test_case::test_case;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Icon::insert_image_tooltip);

		app
	}

	#[test]
	fn insert_tooltip() {
		let handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my text"),
				image: IconImage::Loaded(handle.clone()),
			})
			.id();

		app.update();

		assert_eq!(
			(
				Some(&Tooltip::new(Localized::from("my text"))),
				Some(&Interaction::None)
			),
			(
				app.world().entity(entity).get::<Tooltip<Localized>>(),
				app.world().entity(entity).get::<Interaction>(),
			),
		);
	}

	#[test_case(IconImage::Path(PathBuf::from("")); "is path")]
	#[test_case(IconImage::Loading(new_handle()); "is loading")]
	#[test_case(IconImage::None; "is none")]
	fn do_not_insert_tooltip_when_image(image: IconImage) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my text"),
				image,
			})
			.id();

		app.update();

		assert_eq!(
			(None, None),
			(
				app.world().entity(entity).get::<Tooltip<Localized>>(),
				app.world().entity(entity).get::<Interaction>(),
			),
		);
	}

	#[test]
	fn do_not_insert_tooltip_twice() {
		let handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my text"),
				image: IconImage::Loaded(handle.clone()),
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<(Interaction, Tooltip<Localized>)>();
		app.update();

		assert_eq!(
			(None, None),
			(
				app.world().entity(entity).get::<Tooltip<Localized>>(),
				app.world().entity(entity).get::<Interaction>(),
			),
		);
	}

	#[test]
	fn insert_tooltip_again_after_change() {
		let handle = new_handle();
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Icon {
				localized: Localized::from("my text"),
				image: IconImage::Loaded(handle.clone()),
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<(Interaction, Tooltip<Localized>)>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Icon>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			(
				Some(&Tooltip::new(Localized::from("my text"))),
				Some(&Interaction::None)
			),
			(
				app.world().entity(entity).get::<Tooltip<Localized>>(),
				app.world().entity(entity).get::<Interaction>(),
			),
		);
	}
}
