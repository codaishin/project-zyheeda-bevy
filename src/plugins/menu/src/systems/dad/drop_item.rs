use crate::components::{Dad, KeyedPanel};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{EntityContextMut, TryApplyOn},
		handles_loadout::items::{Items, SwapItems},
	},
	zyheeda_commands::ZyheedaCommands,
};

pub fn drop_item<TAgent, TLoadout>(
	mut commands: ZyheedaCommands,
	agents: Query<(Entity, &Dad), With<TAgent>>,
	panels: Query<(&Interaction, &KeyedPanel)>,
	mouse: Res<ButtonInput<MouseButton>>,
	mut param: StaticSystemParam<TLoadout>,
) where
	TAgent: Component,
	TLoadout: for<'c> EntityContextMut<Items, TContext<'c>: SwapItems>,
{
	if !mouse.just_released(MouseButton::Left) {
		return;
	}

	for (agent, dad) in &agents {
		let Some(mut ctx) = TLoadout::get_entity_context_mut(&mut param, agent, Items) else {
			return;
		};

		for (.., keyed_panel) in panels.iter().filter(is_hovered) {
			ctx.swap_items(dad.0, keyed_panel.0);
			commands.try_apply_on(&agent, |mut e| {
				e.try_remove::<Dad>();
			});
		}
	}
}

fn is_hovered((interaction, ..): &(&Interaction, &KeyedPanel)) -> bool {
	&&Interaction::Hovered == interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ui::Interaction,
	};
	use common::{tools::action_key::slot::SlotKey, traits::handles_loadout::LoadoutKey};
	use testing::{SingleThreadedApp, set_input};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Container {
		swaps: Vec<(LoadoutKey, LoadoutKey)>,
	}

	impl SwapItems for _Container {
		fn swap_items<TA, TB>(&mut self, a: TA, b: TB)
		where
			TA: Into<LoadoutKey>,
			TB: Into<LoadoutKey>,
		{
			self.swaps.push((a.into(), b.into()));
		}
	}

	const MOUSE_LEFT: MouseButton = MouseButton::Left;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(ButtonInput::<MouseButton>::default());
		app.add_systems(Update, drop_item::<_Agent, Query<&mut _Container>>);

		app
	}

	#[test]
	fn call_swap() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Agent, _Container::default(), Dad::from(SlotKey(42))))
			.id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel::from(SlotKey(11))));

		set_input!(app, just_released(MOUSE_LEFT));
		app.update();

		assert_eq!(
			Some(&_Container {
				swaps: vec![(LoadoutKey::from(SlotKey(42)), LoadoutKey::from(SlotKey(11)))]
			}),
			app.world().entity(entity).get::<_Container>(),
		);
	}

	#[test]
	fn do_nothing_when_agent_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Container::default(), Dad::from(SlotKey(42))))
			.id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel::from(SlotKey(11))));

		set_input!(app, just_released(MOUSE_LEFT));
		app.update();

		assert_eq!(
			Some(&_Container { swaps: vec![] }),
			app.world().entity(entity).get::<_Container>(),
		);
	}

	#[test]
	fn do_not_call_swap_when_interaction_not_hover() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Agent, _Container::default(), Dad::from(SlotKey(42))))
			.id();
		app.world_mut()
			.spawn((Interaction::None, KeyedPanel::from(SlotKey(11))));

		set_input!(app, just_released(MOUSE_LEFT));
		app.update();

		assert_eq!(
			Some(&_Container { swaps: vec![] }),
			app.world().entity(entity).get::<_Container>(),
		);
	}

	#[test]
	fn do_not_call_swap_when_mouse_left_not_just_released() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Agent, _Container::default(), Dad::from(SlotKey(42))))
			.id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel::from(SlotKey(11))));

		set_input!(app, released(MOUSE_LEFT));
		app.update();

		assert_eq!(
			Some(&_Container { swaps: vec![] }),
			app.world().entity(entity).get::<_Container>(),
		);
	}

	#[test]
	fn remove_dad() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Agent, _Container::default(), Dad::from(SlotKey(42))))
			.id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel::from(SlotKey(11))));

		set_input!(app, just_released(MOUSE_LEFT));
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Dad>());
	}
}
