use crate::components::dispatch_text_color::DispatchTextColor;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl DispatchTextColor {
	pub(crate) fn apply(
		mut commands: ZyheedaCommands,
		dispatchers: Query<(Entity, &DispatchTextColor), Changed<DispatchTextColor>>,
		texts: Query<(), With<Text>>,
		children: Query<&Children>,
	) {
		for (entity, DispatchTextColor(color)) in &dispatchers {
			for child in children.iter_descendants(entity) {
				insert_color_on_text(&mut commands, child, color, texts);
			}

			insert_color_on_text(&mut commands, entity, color, texts);
		}
	}
}

fn insert_color_on_text(
	commands: &mut ZyheedaCommands,
	entity: Entity,
	color: &Color,
	texts: Query<(), With<Text>>,
) {
	if !texts.contains(entity) {
		return;
	}

	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(TextColor(*color));
	});
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, DispatchTextColor::apply);

		app
	}

	#[test]
	fn dispatch_text_color_to_root_entity() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Text::from(""),
				DispatchTextColor(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&TextColor(Color::srgb(0.1, 0.2, 0.3))),
			app.world().entity(entity).get::<TextColor>(),
		);
	}

	#[test]
	fn do_not_dispatch_text_color_to_root_entity_when_no_text_on_root() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(DispatchTextColor(Color::srgb(0.1, 0.2, 0.3)))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<TextColor>());
	}

	#[test]
	fn dispatch_text_color_to_children() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(DispatchTextColor(Color::srgb(0.1, 0.2, 0.3)))
			.id();
		let child = app
			.world_mut()
			.spawn((Text::from(""), ChildOf(entity)))
			.id();
		let child_child = app.world_mut().spawn((Text::from(""), ChildOf(child))).id();

		app.update();

		assert_eq!(
			[
				Some(&TextColor(Color::srgb(0.1, 0.2, 0.3))),
				Some(&TextColor(Color::srgb(0.1, 0.2, 0.3))),
			],
			app.world()
				.entity([child, child_child])
				.map(|e| e.get::<TextColor>()),
		);
	}

	#[test]
	fn do_not_dispatch_text_color_to_children_with_no_text() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(DispatchTextColor(Color::srgb(0.1, 0.2, 0.3)))
			.id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();
		let child_child = app.world_mut().spawn((Text::from(""), ChildOf(child))).id();

		app.update();

		assert_eq!(
			[None, Some(&TextColor(Color::srgb(0.1, 0.2, 0.3)))],
			app.world()
				.entity([child, child_child])
				.map(|e| e.get::<TextColor>()),
		);
	}

	#[test]
	fn dispatch_text_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Text::from(""),
				DispatchTextColor(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<TextColor>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<TextColor>());
	}

	#[test]
	fn dispatch_text_again_after_change() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Text::from(""),
				DispatchTextColor(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<TextColor>()
			.get_mut::<DispatchTextColor>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&TextColor(Color::srgb(0.1, 0.2, 0.3))),
			app.world().entity(entity).get::<TextColor>(),
		);
	}
}
