use crate::components::{Dad, KeyedPanel};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_loadout::loadout::{LoadoutKey, SwapExternal},
	},
	zyheeda_commands::ZyheedaCommands,
};

pub fn drop_item_external<TContainerA, TContainerB>(
	mut commands: ZyheedaCommands,
	mut agents: Query<(
		Entity,
		&mut TContainerA,
		&mut TContainerB,
		&Dad<TContainerA::TKey>,
	)>,
	panels: Query<(&Interaction, &KeyedPanel<TContainerB::TKey>)>,
	mouse: Res<ButtonInput<MouseButton>>,
) where
	TContainerA: Component<Mutability = Mutable> + SwapExternal<TContainerB>,
	TContainerB: Component<Mutability = Mutable> + LoadoutKey,
{
	if !mouse.just_released(MouseButton::Left) {
		return;
	}

	let Ok((entity, mut container_a, mut container_b, dad)) = agents.single_mut() else {
		return;
	};

	let Some((.., keyed_panel)) = panels.iter().find(is_hovered) else {
		return;
	};

	container_a.swap_external(container_b.as_mut(), dad.0, keyed_panel.0);
	commands.try_apply_on(&entity, |mut e| {
		e.try_remove::<Dad<TContainerA::TKey>>();
	});
}

fn is_hovered<TDadPanel>((interaction, ..): &(&Interaction, &KeyedPanel<TDadPanel>)) -> bool {
	&&Interaction::Hovered == interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ui::Interaction,
	};
	use common::tools::action_key::slot::{PlayerSlot, Side};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, set_input};

	#[derive(Component, NestedMocks)]
	struct _ContainerA {
		mock: Mock_ContainerA,
	}

	impl Default for _ContainerA {
		fn default() -> Self {
			let mut mock = Mock_ContainerA::default();
			mock.expect_swap_external::<usize, f32>().return_const(());

			Self { mock }
		}
	}

	impl LoadoutKey for _ContainerA {
		type TKey = usize;
	}

	impl LoadoutKey for Mock_ContainerA {
		type TKey = usize;
	}

	#[automock]
	impl SwapExternal<_ContainerB> for _ContainerA {
		fn swap_external<TKey, TOtherKey>(&mut self, other: &mut _ContainerB, a: TKey, b: TOtherKey)
		where
			TKey: Into<usize> + 'static,
			TOtherKey: Into<f32> + 'static,
		{
			self.mock.swap_external(other, a, b);
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ContainerB;

	impl LoadoutKey for _ContainerB {
		type TKey = f32;
	}

	const MOUSE_LEFT: MouseButton = MouseButton::Left;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(ButtonInput::<MouseButton>::default());
		app.add_systems(Update, drop_item_external::<_ContainerA, _ContainerB>);

		app
	}

	#[test]
	fn call_swap() {
		let mut app = setup();
		app.world_mut().spawn((
			_ContainerA::new().with_mock(|mock| {
				mock.expect_swap_external::<usize, f32>()
					.with(eq(_ContainerB), eq(42), eq(11.))
					.times(1)
					.return_const(());
			}),
			_ContainerB,
			Dad(42_usize),
		));
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn no_panic_when_agent_has_no_dad() {
		let mut app = setup();
		app.world_mut().spawn((_ContainerA::default(), _ContainerB));
		app.world_mut().spawn((
			Interaction::Hovered,
			KeyedPanel(PlayerSlot::Upper(Side::Left)),
		));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(11_f32)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn do_not_call_swap_when_interaction_not_hover() {
		let mut app = setup();
		app.world_mut().spawn((
			_ContainerA::new().with_mock(|mock| {
				mock.expect_swap_external::<usize, f32>().never();
			}),
			_ContainerB,
			Dad(42_usize),
		));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(11_f32)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn do_not_call_swap_when_not_mouse_left_release() {
		let mut app = setup();
		app.world_mut().spawn((
			_ContainerA::new().with_mock(|mock| {
				mock.expect_swap_external::<usize, f32>().never();
			}),
			_ContainerB,
			Dad(42_usize),
		));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(11_f32)));
		set_input!(app, pressed(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn remove_dad() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_ContainerA::default(), _ContainerB, Dad(42_usize)))
			.id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_f32)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();

		assert!(!app.world().entity(entity).contains::<Dad<usize>>());
	}
}
