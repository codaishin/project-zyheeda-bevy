use crate::{
	components::dropdown::{DropdownActive, DropdownContainerOf, IsDropdown},
	resources::dropdown_change::DropdownChanged,
};
use bevy::prelude::*;
use common::zyheeda_commands::ZyheedaCommands;
use std::ops::DerefMut;

impl DropdownActive {
	pub(crate) fn set_activity(
		pressed: On<PointerPress>,
		commands: ZyheedaCommands,
		mut dropdowns: Query<Option<&mut DropdownActive>, With<IsDropdown>>,
		mut changed: ResMut<DropdownChanged>,
	) {
		if pressed.button != PointerButton::Primary {
			return;
		}

		**changed = true;

		let Ok(active) = dropdowns.get_mut(pressed.entity) else {
			return;
		};

		match active {
			Some(active) => refresh(active),
			None => activate(pressed, commands),
		}
	}
}

fn refresh(mut active: Mut<DropdownActive>) {
	active.deref_mut();
}

fn activate(pressed: On<PointerPress>, mut commands: ZyheedaCommands) {
	commands.spawn((ChildOf(pressed.entity), DropdownContainerOf(pressed.entity)));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::dropdown::{DropdownContainerOf, IsDropdown};
	use bevy::{
		camera::{ImageRenderTarget, NormalizedRenderTarget},
		picking::{
			backend::HitData,
			pointer::{Location, PointerId},
		},
	};
	use testing::{IsChanged, SingleThreadedApp, assert_related_count, fake_entity, new_handle};

	fn pointer(button: PointerButton) -> impl Fn(Entity) -> PointerPress {
		move |entity| PointerPress {
			entity,
			pointer: Pointer::new(
				PointerId::Mouse,
				Location {
					target: NormalizedRenderTarget::Image(ImageRenderTarget {
						handle: new_handle(),
						scale_factor: 1.,
					}),
					position: Vec2::ZERO,
				},
			),
			button,
			hit: HitData {
				camera: fake_entity!(42),
				depth: 1.,
				extra: None,
				position: None,
				normal: None,
			},
			count: 1,
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<DropdownChanged>();
		app.add_observer(DropdownActive::set_activity);
		app.add_systems(Update, IsChanged::<DropdownActive>::detect);

		app
	}

	mod on_primary {
		use super::*;

		#[test]
		fn set_dropdown_changed() {
			let mut app = setup();
			app.world_mut()
				.spawn(IsDropdown)
				.trigger(pointer(PointerButton::Primary));

			app.update();

			assert!(app.world().resource::<DropdownChanged>());
		}

		#[test]
		fn set_dropdown_changed_when_no_dropdown_present() {
			let mut app = setup();
			app.world_mut()
				.spawn(())
				.trigger(pointer(PointerButton::Primary));

			app.update();

			assert!(app.world().resource::<DropdownChanged>());
		}

		#[test]
		fn activate() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(IsDropdown)
				.trigger(pointer(PointerButton::Primary))
				.id();

			app.update();

			let [child] = assert_related_count!(DropdownContainerOf, 1, app, entity);
			assert_eq!(Some(&ChildOf(entity)), child.get::<ChildOf>());
		}

		#[test]
		fn do_not_re_activate() {
			let mut app = setup();
			let entity = app.world_mut().spawn(IsDropdown).id();
			let original = app
				.world_mut()
				.spawn((ChildOf(entity), DropdownContainerOf(entity)))
				.id();

			app.world_mut()
				.entity_mut(entity)
				.trigger(pointer(PointerButton::Primary));
			app.update();

			let [child] = assert_related_count!(DropdownContainerOf, 1, app, entity,);
			assert_eq!(original, child.id());
		}

		#[test]
		fn refresh_activated() {
			let mut app = setup();
			let entity = app.world_mut().spawn(IsDropdown).id();
			app.world_mut()
				.spawn((ChildOf(entity), DropdownContainerOf(entity)));

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.trigger(pointer(PointerButton::Primary));
			app.update();

			assert_eq!(
				Some(&IsChanged::TRUE),
				app.world()
					.entity(entity)
					.get::<IsChanged<DropdownActive>>(),
			);
		}

		#[test]
		fn do_not_set_activate_on_non_dropdown() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn_empty()
				.trigger(pointer(PointerButton::Primary))
				.id();

			app.update();

			assert_related_count!(DropdownContainerOf, 0, app, entity);
		}
	}

	mod non_primary {
		use super::*;
		use test_case::test_case;

		#[test_case(PointerButton::Secondary; "on secondary")]
		#[test_case(PointerButton::Middle; "on middle")]
		fn do_not_set_dropdown_changed(button: PointerButton) {
			let mut app = setup();
			app.world_mut().spawn(IsDropdown).trigger(pointer(button));

			app.update();

			assert!(!app.world().resource::<DropdownChanged>());
		}

		#[test_case(PointerButton::Secondary; "on secondary")]
		#[test_case(PointerButton::Middle; "on middle")]
		fn do_not_expand(button: PointerButton) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(IsDropdown)
				.trigger(pointer(button))
				.id();

			app.update();

			assert_related_count!(DropdownContainerOf, 0, app, entity);
		}
	}
}
