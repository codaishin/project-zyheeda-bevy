use crate::components::{Dad, KeyedPanel};
use bevy::{
	ecs::{
		component::Component,
		system::{Commands, Query},
	},
	input::ButtonInput,
	prelude::{Entity, MouseButton, Res},
	ui::Interaction,
};
use common::traits::{
	handles_loadout_menus::{SwapKey, SwapValuesByKey},
	thread_safe::ThreadSafe,
	try_remove_from::TryRemoveFrom,
};

pub fn drop<TAgent, TKeyDad, TKeyKeyedPanel>(
	mut commands: Commands,
	mut agents: Query<(Entity, &mut TAgent, &Dad<TKeyDad>)>,
	panels: Query<(&Interaction, &KeyedPanel<TKeyKeyedPanel>)>,
	mouse: Res<ButtonInput<MouseButton>>,
) where
	TAgent: Component + SwapValuesByKey,
	TKeyDad: ThreadSafe + Copy + Into<SwapKey>,
	TKeyKeyedPanel: ThreadSafe + Copy + Into<SwapKey>,
{
	if !mouse.just_released(MouseButton::Left) {
		return;
	}

	let Ok((entity, mut agent, dad)) = agents.get_single_mut() else {
		return;
	};

	let Some((.., keyed_panel)) = panels.iter().find(is_hovered) else {
		return;
	};

	agent.swap(dad.0.into(), keyed_panel.0.into());
	commands.try_remove_from::<Dad<TKeyDad>>(entity);
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
		test_tools::utils::SingleThreadedApp,
		tools::{
			inventory_key::InventoryKey,
			slot_key::{Side, SlotKey},
		},
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

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
		app.insert_resource(ButtonInput::<MouseButton>::default());
		app.add_systems(Update, drop::<_Agent, InventoryKey, SlotKey>);

		app
	}

	macro_rules! press_and_release_mouse_left {
		($app:expr) => {{
			let mut input = $app
				.world_mut()
				.get_resource_mut::<ButtonInput<MouseButton>>()
				.unwrap();

			input.press(MouseButton::Left);
			input.release(MouseButton::Left);
		}};
	}

	#[test]
	fn call_swap() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent::new().with_mock(|mock| {
				mock.expect_swap()
					.with(
						eq(SwapKey::Inventory(InventoryKey(42))),
						eq(SwapKey::Slot(SlotKey::TopHand(Side::Left))),
					)
					.times(1)
					.return_const(());
			}),
			Dad(InventoryKey(42)),
		));

		press_and_release_mouse_left!(app);
		app.world_mut().spawn((
			Interaction::Hovered,
			KeyedPanel(SlotKey::TopHand(Side::Left)),
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
			KeyedPanel(SlotKey::TopHand(Side::Left)),
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
			KeyedPanel(SlotKey::TopHand(Side::Left)),
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
			KeyedPanel(SlotKey::TopHand(Side::Left)),
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
			KeyedPanel(SlotKey::TopHand(Side::Left)),
		));
		app.update();

		assert!(!app.world().entity(agent).contains::<Dad<usize>>());
	}
}
