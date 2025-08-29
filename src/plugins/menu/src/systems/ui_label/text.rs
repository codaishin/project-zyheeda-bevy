use crate::components::{
	icon::Icon,
	label::{UILabel, UILabelText},
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		iter_descendants_conditional::IterDescendantsConditional,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl UILabel {
	pub(crate) fn text(
		mut commands: ZyheedaCommands,
		icons: Query<(Entity, &Icon, &UILabel), IconOrLabelChanged>,
		children: Query<&Children>,
		markers: Query<(), With<UILabelText>>,
		labels: Query<(), With<UILabel>>,
	) {
		for (entity, icon, UILabel(label)) in &icons {
			if icon.has_image() {
				continue;
			}

			let child = children
				.iter_descendants_conditional(entity, |e| !labels.contains(e))
				.find(|e| markers.contains(*e));

			commands.try_apply_on(&child.unwrap_or(entity), |mut e| {
				e.try_insert(Text::from(label.clone()));
			});
		}
	}
}

type IconOrLabelChanged = Or<(Changed<Icon>, Changed<UILabel>)>;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::label::UILabelText;
	use common::traits::handles_localization::localized::Localized;
	use std::path::PathBuf;
	use test_case::test_case;
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, UILabel::text);

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

	#[test]
	fn insert_text_on_child_with_label_marker() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((UILabel(Localized::from("my description")), Icon::None))
			.id();
		let child = app.world_mut().spawn((UILabelText, ChildOf(entity))).id();

		app.update();

		assert_eq!(
			Some("my description"),
			app.world()
				.entity(child)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		);
	}

	#[test]
	fn insert_text_on_deep_child_with_label_marker() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(UILabel(Localized::from("my description")))
			.id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();
		let deep_child = app.world_mut().spawn((UILabelText, ChildOf(child))).id();

		app.update();

		assert_eq!(
			Some("my description"),
			app.world()
				.entity(deep_child)
				.get::<Text>()
				.map(|Text(text)| text.as_str())
		);
	}

	#[test]
	fn match_label_markers_with_labels() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(UILabel(Localized::from("my description")))
			.id();
		let child = app
			.world_mut()
			.spawn((
				UILabel(Localized::from("my description 2")),
				ChildOf(entity),
			))
			.id();
		let deep_child = app.world_mut().spawn((UILabelText, ChildOf(child))).id();

		app.update();

		assert_eq!(
			(Some("my description"), Some("my description 2")),
			(
				app.world()
					.entity(entity)
					.get::<Text>()
					.map(|Text(text)| text.as_str()),
				app.world()
					.entity(deep_child)
					.get::<Text>()
					.map(|Text(text)| text.as_str()),
			)
		);
	}

	#[test_case(Icon::ImagePath(PathBuf::from("")); "is path")]
	#[test_case(Icon::Load(new_handle()); "is loading")]
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
