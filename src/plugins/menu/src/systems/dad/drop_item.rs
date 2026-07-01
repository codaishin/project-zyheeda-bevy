use crate::components::{Dad, KeyedPanel};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{Get, TryApplyOn, TryGetContextMut, View},
		handles_loadout::items::{Items, SwapItems},
		handles_player::PlayerEntity,
	},
	zyheeda_commands::ZyheedaCommands,
};

pub fn drop_item<TPlayer, TLoadout>(
	mut commands: ZyheedaCommands,
	dads: Query<&Dad>,
	player: StaticSystemParam<TPlayer>,
	panels: Query<(&Interaction, &KeyedPanel)>,
	mouse: Res<ButtonInput<MouseButton>>,
	mut param: StaticSystemParam<TLoadout>,
) where
	TPlayer: for<'w, 's> SystemParam<Item<'w, 's>: View<PlayerEntity>>,
	TLoadout: for<'c> TryGetContextMut<Items, TContext<'c>: SwapItems>,
{
	if !mouse.just_released(MouseButton::Left) {
		return;
	}

	let Some(player) = player.view() else {
		return;
	};
	let Some(entity) = commands.get(&player) else {
		return;
	};

	let Some(mut ctx) = TLoadout::try_get_context_mut(&mut param, Items { entity }) else {
		return;
	};

	let Ok(dad) = dads.get(entity) else {
		return;
	};

	for (.., keyed_panel) in panels.iter().filter(is_hovered) {
		ctx.swap_items(dad.0, keyed_panel.0);
		commands.try_apply_on(&entity, |mut e| {
			e.try_remove::<Dad>();
		});
	}
}

fn is_hovered((interaction, ..): &(&Interaction, &KeyedPanel)) -> bool {
	&&Interaction::Hovered == interaction
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::testing::{_Player, _PlayerParam};
	use bevy::{
		app::{App, Update},
		ui::Interaction,
	};
	use common::{
		CommonPlugin,
		tools::action_key::slot::SlotKey,
		traits::handles_loadout::LoadoutKey,
	};
	use testing::{SingleThreadedApp, set_input};

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

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.insert_resource(ButtonInput::<MouseButton>::default());
		app.add_systems(Update, drop_item::<_PlayerParam, Query<&mut _Container>>);

		app
	}

	#[test]
	fn call_swap() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((_Player, _Container::default(), Dad::from(SlotKey(42))))
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
			.spawn((_Player, _Container::default(), Dad::from(SlotKey(42))))
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
			.spawn((_Player, _Container::default(), Dad::from(SlotKey(42))))
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
			.spawn((_Player, _Container::default(), Dad::from(SlotKey(42))))
			.id();
		app.world_mut()
			.spawn((Interaction::Hovered, KeyedPanel::from(SlotKey(11))));

		set_input!(app, just_released(MOUSE_LEFT));
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Dad>());
	}
}
