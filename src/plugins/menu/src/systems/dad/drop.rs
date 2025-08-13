use crate::components::{Dad, KeyedPanel};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	tools::{action_key::user_input::UserInput, swap_key::SwapKey},
	traits::{
		accessors::get::TryApplyOn,
		handles_loadout_menu::SwapValuesByKey,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

pub fn drop<TAgent, TKeyDad, TKeyKeyedPanel>(
	mut commands: ZyheedaCommands,
	mut agents: Query<(Entity, &mut TAgent, &Dad<TKeyDad>)>,
	panels: Query<(&Interaction, &KeyedPanel<TKeyKeyedPanel>)>,
	mouse: Res<ButtonInput<UserInput>>,
) where
	TAgent: Component<Mutability = Mutable> + SwapValuesByKey,
	TKeyDad: ThreadSafe + Copy + Into<SwapKey>,
	TKeyKeyedPanel: ThreadSafe + Copy + Into<SwapKey>,
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

	agent.swap(dad.0.into(), keyed_panel.0.into());
	commands.try_apply_on(&entity, |mut e| {
		e.try_remove::<Dad<TKeyDad>>();
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
	use common::tools::{
		action_key::slot::{PlayerSlot, Side},
		inventory_key::InventoryKey,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Agent {
		mock: Mock_Agent,
	}

	impl Default for _Agent {
		fn default() -> Self {
			let mut mock = Mock_Agent::default();
			mock.expect_swap().return_const(());

			Self { mock }
		}
	}

	#[automock]
	impl SwapValuesByKey for _Agent {
		fn swap(&mut self, a: SwapKey, b: SwapKey) {
			self.mock.swap(a, b);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(ButtonInput::<UserInput>::default());
		app.add_systems(Update, drop::<_Agent, InventoryKey, PlayerSlot>);

		app
	}

	macro_rules! press_and_release_mouse_left {
		($app:expr) => {{
			let mut input = $app
				.world_mut()
				.get_resource_mut::<ButtonInput<UserInput>>()
				.unwrap();

			input.press(UserInput::from(MouseButton::Left));
			input.release(UserInput::from(MouseButton::Left));
		}};
	}

	#[test]
	fn call_swap() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent::new().with_mock(|mock| {
				mock.expect_swap()
					.with(
						eq(SwapKey::from(InventoryKey(42))),
						eq(SwapKey::from(PlayerSlot::Upper(Side::Left))),
					)
					.times(1)
					.return_const(());
			}),
			Dad(InventoryKey(42)),
		));

		press_and_release_mouse_left!(app);
		app.world_mut().spawn((
			Interaction::Hovered,
			KeyedPanel(PlayerSlot::Upper(Side::Left)),
		));

		app.update();
	}

	#[test]
	fn no_panic_when_agent_has_no_dad() {
		let mut app = setup();
		app.world_mut().spawn(_Agent::default());

		press_and_release_mouse_left!(app);
		app.world_mut().spawn((
			Interaction::Hovered,
			KeyedPanel(PlayerSlot::Upper(Side::Left)),
		));
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel(22222_f32)));

		app.update();
	}

	#[test]
	fn do_not_call_swap_when_interaction_not_hover() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent::new().with_mock(|mock| {
				mock.expect_swap().never().return_const(());
			}),
			Dad(InventoryKey(42)),
		));

		press_and_release_mouse_left!(app);
		app.world_mut().spawn((
			Interaction::Pressed,
			KeyedPanel(PlayerSlot::Upper(Side::Left)),
		));

		app.update();
	}

	#[test]
	fn do_not_call_swap_when_not_mouse_left_release() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent::new().with_mock(|mock| {
				mock.expect_swap().never().return_const(());
			}),
			Dad(InventoryKey(42)),
		));

		app.world_mut().spawn((
			Interaction::Hovered,
			KeyedPanel(PlayerSlot::Upper(Side::Left)),
		));

		app.update();
	}

	#[test]
	fn remove_dad() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((_Agent::default(), Dad(InventoryKey(42))))
			.id();

		press_and_release_mouse_left!(app);
		app.world_mut().spawn((
			Interaction::Hovered,
			KeyedPanel(PlayerSlot::Upper(Side::Left)),
		));
		app.update();

		assert!(!app.world().entity(agent).contains::<Dad<usize>>());
	}
}
