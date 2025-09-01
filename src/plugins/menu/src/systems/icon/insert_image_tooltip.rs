use crate::{
	Tooltip,
	components::{icon::Icon, label::UILabel},
};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl Icon {
	pub(crate) fn insert_image_tooltip(
		mut commands: ZyheedaCommands,
		icons: Query<(Entity, &Self, &UILabel), Changed<Self>>,
	) {
		for (entity, icon, UILabel(label)) in &icons {
			if !matches!(icon, Icon::Loaded(_)) {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert((Interaction::default(), Tooltip::new(label.clone())));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_localization::localized::Localized;
	use std::path::PathBuf;
	use test_case::test_case;
	use testing::{SingleThreadedApp, new_handle};

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
			.spawn((
				UILabel(Localized::from("my text")),
				Icon::Loaded(handle.clone()),
			))
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

	#[test_case(Icon::ImagePath(PathBuf::from("")); "is path")]
	#[test_case(Icon::Loading(new_handle()); "is loading")]
	#[test_case(Icon::None; "is none")]
	fn do_not_insert_tooltip_icon_when_image(icon: Icon) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UILabel(Localized::from("my text")), icon))
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
			.spawn((
				UILabel(Localized::from("my text")),
				Icon::Loaded(handle.clone()),
			))
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
			.spawn((
				UILabel(Localized::from("my text")),
				Icon::Loaded(handle.clone()),
			))
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
