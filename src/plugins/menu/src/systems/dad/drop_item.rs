use crate::components::{Dad, KeyedPanel};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	tools::action_key::user_input::UserInput,
	traits::{accessors::get::TryApplyOn, handles_loadout::SwapInternal},
	zyheeda_commands::ZyheedaCommands,
};

pub fn drop_item<TContainer>(
	mut commands: ZyheedaCommands,
	mut agents: Query<(Entity, &mut TContainer, &Dad<TContainer::TKey>)>,
	panels: Query<(&Interaction, &KeyedPanel<TContainer::TKey>)>,
	mouse: Res<ButtonInput<UserInput>>,
) where
	TContainer: Component<Mutability = Mutable> + SwapInternal,
{
	if !mouse.just_released(UserInput::from(MouseButton::Left)) {
		return;
	}

	let Ok((entity, mut agent, dad)) = agents.single_mut() else {
		return;
	};

	let Some((.., keyed_panel)) = panels.iter().find(is_hovered) else {
		return;
	};

	agent.swap_internal(dad.0, keyed_panel.0);
	commands.try_apply_on(&entity, |mut e| {
		e.try_remove::<Dad<TContainer::TKey>>();
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
	use common::{
		tools::action_key::slot::{PlayerSlot, Side},
		traits::handles_loadout::ContainerKey,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, set_input};

	#[derive(Component, NestedMocks)]
	struct _Container {
		mock: Mock_Container,
	}

	impl Default for _Container {
		fn default() -> Self {
			let mut mock = Mock_Container::default();
			mock.expect_swap_internal::<usize>().return_const(());

			Self { mock }
		}
	}

	impl ContainerKey for _Container {
		type TKey = usize;
	}

	impl ContainerKey for Mock_Container {
		type TKey = usize;
	}

	#[automock]
	impl SwapInternal for _Container {
		fn swap_internal<TKey>(&mut self, a: TKey, b: TKey)
		where
			TKey: Into<usize> + 'static,
		{
			self.mock.swap_internal(a, b);
		}
	}

	const MOUSE_LEFT: UserInput = UserInput::MouseButton(MouseButton::Left);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(ButtonInput::<UserInput>::default());
		app.add_systems(Update, drop_item::<_Container>);

		app
	}

	#[test]
	fn call_swap() {
		let mut app = setup();
		app.world_mut().spawn((
			_Container::new().with_mock(|mock| {
				mock.expect_swap_internal::<usize>()
					.with(eq(42), eq(11))
					.times(1)
					.return_const(());
			}),
			Dad(42_usize),
		));
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_usize)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn no_panic_when_agent_has_no_dad() {
		let mut app = setup();
		app.world_mut().spawn(_Container::default());
		app.world_mut().spawn((
			Interaction::Hovered,
			KeyedPanel(PlayerSlot::Upper(Side::Left)),
		));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(11_usize)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn do_not_call_swap_when_interaction_not_hover() {
		let mut app = setup();
		app.world_mut().spawn((
			_Container::new().with_mock(|mock| {
				mock.expect_swap_internal::<usize>().never();
			}),
			Dad(42_usize),
		));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(11_usize)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn do_not_call_swap_when_not_mouse_left_release() {
		let mut app = setup();
		app.world_mut().spawn((
			_Container::new().with_mock(|mock| {
				mock.expect_swap_internal::<usize>().never();
			}),
			Dad(42_usize),
		));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(11_usize)));
		set_input!(app, pressed(MOUSE_LEFT));

		app.update();
	}

	#[test]
	fn remove_dad() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Container::default(), Dad(42_usize)))
			.id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel(11_usize)));
		set_input!(app, pressed(MOUSE_LEFT));
		set_input!(app, just_released(MOUSE_LEFT));

		app.update();

		assert!(!app.world().entity(entity).contains::<Dad<usize>>());
	}
}
