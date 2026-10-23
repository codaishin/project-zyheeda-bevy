use crate::components::combo_skill_button::{ComboSkillButton, DropdownItem};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
	ui_widgets::Activate,
};
use common::prelude::*;
use std::fmt::Debug;

impl<TLayout, TId> ComboSkillButton<DropdownItem<TLayout>, TId>
where
	TLayout: 'static,
	TId: Debug + PartialEq + Clone + ThreadSafe,
{
	pub(crate) fn update<TPlayer, TLoadout>(
		activated: On<Activate>,
		commands: ZyheedaCommands,
		skill_buttons: Query<&Self>,
		player: StaticSystemParam<TPlayer>,
		mut param: StaticSystemParam<TLoadout>,
	) where
		TPlayer: for<'w, 's> SystemParam<Item<'w, 's>: View<PlayerEntity>>,
		TLoadout: for<'c> TryGetContextMut<Combos, TContext<'c>: UpdateCombos<TId>>,
	{
		let Ok(skill_button) = skill_buttons.get(activated.entity) else {
			return;
		};

		let Some(player) = player.view() else {
			return;
		};

		let Some(entity) = commands.get(&player) else {
			return;
		};

		let Some(mut ctx) = TLoadout::try_get_context_mut(&mut param, Combos { entity }) else {
			return;
		};

		ctx.update_combos(vec![(
			skill_button.key_path.clone(),
			Some(skill_button.skill.id.clone()),
		)]);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::combo_overview::ComboSkill,
		testing::{_Player, _PlayerParam},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::sync::LazyLock;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Combos {
		mock: Mock_Combos,
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _Id;

	#[automock]
	impl UpdateCombos<_Id> for _Combos {
		fn update_combos(&mut self, combos: Combo<SlotKey, Option<_Id>>) {
			self.mock.update_combos(combos);
		}
	}

	struct _Layout;

	static SKILL: LazyLock<ComboSkill<_Id>> = LazyLock::new(|| ComboSkill {
		id: _Id,
		token: Token::default(),
		icon: Handle::default(),
	});

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.add_observer(
			ComboSkillButton::<DropdownItem<_Layout>, _Id>::update::<
				_PlayerParam,
				Query<&mut _Combos>,
			>,
		);

		app
	}

	#[test]
	fn update_skill() {
		let mut app = setup();

		app.world_mut().spawn((
			_Player,
			_Combos::new().with_mock(|mock| {
				mock.expect_update_combos()
					.times(1)
					.with(eq(vec![(vec![SlotKey::from(HandSlot::Left)], Some(_Id))]))
					.return_const(());
			}),
		));
		app.world_mut()
			.spawn(ComboSkillButton::<DropdownItem<_Layout>, _Id>::new(
				SKILL.clone(),
				vec![SlotKey::from(HandSlot::Left)],
			))
			.trigger(|entity| Activate { entity });
	}

	#[test]
	fn do_not_update_skill_when_interaction_not_pressed() {
		let mut app = setup();

		app.world_mut().spawn((
			_Player,
			_Combos::new().with_mock(|mock| {
				mock.expect_update_combos().never();
			}),
		));
		app.world_mut()
			.spawn(ComboSkillButton::<DropdownItem<_Layout>, _Id>::new(
				SKILL.clone(),
				vec![SlotKey::from(HandSlot::Left)],
			));
	}

	#[test]
	fn do_not_update_skill_when_agent_missing_interaction_not_pressed() {
		let mut app = setup();
		app.world_mut().spawn(_Combos::new().with_mock(|mock| {
			mock.expect_update_combos().never();
		}));
		app.world_mut()
			.spawn(ComboSkillButton::<DropdownItem<_Layout>, _Id>::new(
				SKILL.clone(),
				vec![SlotKey::from(HandSlot::Left)],
			))
			.trigger(|entity| Activate { entity });
	}
}
