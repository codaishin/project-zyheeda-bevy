use crate::components::dropdown::DropdownState;
use bevy::{platform::collections::HashSet, prelude::*};

impl DropdownState {
	pub(crate) fn set(
		pressed: On<PointerPress>,
		mut dropdowns: Query<(Entity, &mut DropdownState)>,
		children: Query<&ChildOf>,
	) {
		if pressed.button != PointerButton::Primary {
			return;
		}

		let mut already_set = HashSet::from([]);

		for ancestor in children.iter_ancestors(pressed.entity) {
			let Ok((_, state)) = dropdowns.get_mut(ancestor) else {
				continue;
			};

			Self::update(state, DropdownState::Expanded);
			already_set.insert(ancestor);
		}

		for (entity, state) in &mut dropdowns {
			if already_set.contains(&entity) {
				continue;
			}

			let new_state = if entity == pressed.entity {
				DropdownState::Expanded
			} else {
				DropdownState::Collapsed
			};

			Self::update(state, new_state);
		}
	}

	fn update(mut this: Mut<Self>, new: Self) {
		if *this == new {
			return;
		}

		*this = new;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		camera::{ImageRenderTarget, NormalizedRenderTarget},
		picking::{
			backend::HitData,
			pointer::{Location, PointerId},
		},
	};
	use testing::{IsChanged, SingleThreadedApp, fake_entity, new_handle};

	struct _T;

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

		app.add_observer(DropdownState::set);
		app.add_systems(Update, IsChanged::<DropdownState>::detect);

		app
	}

	mod on_primary {
		use super::*;

		#[test]
		fn expand() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(DropdownState::Collapsed)
				.trigger(pointer(PointerButton::Primary))
				.id();

			app.update();

			assert_eq!(
				Some(&DropdownState::Expanded),
				app.world().entity(entity).get::<DropdownState>(),
			);
		}

		#[test]
		fn do_not_re_expand() {
			let mut app = setup();
			let entity = app.world_mut().spawn(DropdownState::Expanded).id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.trigger(pointer(PointerButton::Primary));
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world().entity(entity).get::<IsChanged<DropdownState>>(),
			);
		}

		#[test]
		fn do_not_set_expand_on_non_dropdown() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn_empty()
				.trigger(pointer(PointerButton::Primary))
				.id();

			app.update();

			assert_eq!(None, app.world().entity(entity).get::<DropdownState>());
		}

		#[test]
		fn collapse_other_dropdowns() {
			let mut app = setup();
			let other = app.world_mut().spawn(DropdownState::Expanded).id();
			app.world_mut()
				.spawn(DropdownState::Collapsed)
				.trigger(pointer(PointerButton::Primary));

			app.update();

			assert_eq!(
				Some(&DropdownState::Collapsed),
				app.world().entity(other).get::<DropdownState>(),
			);
		}

		#[test]
		fn expand_ancestor() {
			let mut app = setup();
			let ancestor = app.world_mut().spawn(DropdownState::Collapsed).id();
			let intermediate = app.world_mut().spawn(ChildOf(ancestor)).id();
			app.world_mut()
				.spawn((ChildOf(intermediate), DropdownState::Collapsed))
				.trigger(pointer(PointerButton::Primary));

			app.update();

			assert_eq!(
				Some(&DropdownState::Expanded),
				app.world().entity(ancestor).get::<DropdownState>(),
			);
		}

		#[test]
		fn do_not_re_expand_ancestor() {
			let mut app = setup();
			let ancestor = app.world_mut().spawn(DropdownState::Expanded).id();
			let intermediate = app.world_mut().spawn(ChildOf(ancestor)).id();
			let entity = app
				.world_mut()
				.spawn((ChildOf(intermediate), DropdownState::Collapsed))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.trigger(pointer(PointerButton::Primary));
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world()
					.entity(ancestor)
					.get::<IsChanged<DropdownState>>(),
			);
		}

		#[test]
		fn do_not_expand_non_dropdown_ancestors() {
			let mut app = setup();
			let ancestor = app.world_mut().spawn(DropdownState::Collapsed).id();
			let intermediate = app.world_mut().spawn(ChildOf(ancestor)).id();
			app.world_mut()
				.spawn((ChildOf(intermediate), DropdownState::Collapsed))
				.trigger(pointer(PointerButton::Primary));

			app.update();

			assert_eq!(
				None,
				app.world().entity(intermediate).get::<DropdownState>(),
			);
		}
	}

	mod non_primary {
		use super::*;
		use test_case::test_case;

		#[test_case(PointerButton::Secondary)]
		#[test_case(PointerButton::Middle)]
		fn do_not_expand(button: PointerButton) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(DropdownState::Collapsed)
				.trigger(pointer(button))
				.id();

			app.update();

			assert_eq!(
				Some(&DropdownState::Collapsed),
				app.world().entity(entity).get::<DropdownState>(),
			);
		}
	}
}
