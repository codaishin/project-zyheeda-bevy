use crate::traits::{UpdateCombosView, build_combo_tree_layout::BuildComboTreeLayout};
use bevy::{
	ecs::{
		component::Mutable,
		system::{StaticSystemParam, SystemParam},
	},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{ContextChanged, Get, TryGetContext, View},
		handles_loadout::combos::Combos,
		handles_player::PlayerEntity,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Debug;

impl<T> UpdateComboOverview for T where T: Component<Mutability = Mutable> {}

pub(crate) trait UpdateComboOverview: Component<Mutability = Mutable> + Sized {
	fn update_from<TPlayer, TLoadout, TId>(
		commands: ZyheedaCommands,
		mut combo_overviews: Query<&mut Self>,
		player: StaticSystemParam<TPlayer>,
		param: StaticSystemParam<TLoadout>,
	) where
		Self: UpdateCombosView<TId>,
		TPlayer: for<'w, 's> SystemParam<Item<'w, 's>: View<PlayerEntity>>,
		TLoadout: for<'c> TryGetContext<Combos, TContext<'c>: BuildComboTreeLayout<TId>>,
		TId: Debug + PartialEq + Clone,
	{
		let Some(player) = player.view() else {
			return;
		};
		let Some(entity) = commands.get(&player) else {
			return;
		};
		let Some(ctx) = TLoadout::try_get_context(&param, Combos { entity }) else {
			return;
		};

		for mut combo_overview in &mut combo_overviews {
			if !ctx.context_changed() && !combo_overview.is_added() {
				continue;
			}

			combo_overview.update_combos_view(ctx.build_combo_tree_layout());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::combo_overview::ComboSkill,
		testing::{_Player, _PlayerParam},
		traits::build_combo_tree_layout::{ComboTreeElement, ComboTreeLayout, Symbol},
	};
	use bevy::ecs::system::SystemParam;
	use common::{CommonPlugin, tools::action_key::slot::SlotKey};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks, Debug)]
	struct _ComboOverview {
		mock: Mock_ComboOverview,
	}

	#[automock]
	impl UpdateCombosView<_Id> for _ComboOverview {
		fn update_combos_view(&mut self, combos: ComboTreeLayout<SlotKey, ComboSkill<_Id>>) {
			self.mock.update_combos_view(combos)
		}
	}

	#[derive(SystemParam)]
	struct _Param<'w, 's>(Query<'w, 's, Ref<'static, _Combos>>);

	impl TryGetContext<Combos> for _Param<'static, 'static> {
		type TContext<'ctx> = _CombosContext<'ctx>;

		fn try_get_context<'ctx>(
			param: &'ctx _Param,
			Combos { entity }: Combos,
		) -> Option<Self::TContext<'ctx>> {
			param.0.get(entity).ok().map(_CombosContext)
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _Id;

	#[derive(Component, Clone)]
	struct _Combos(ComboTreeLayout<SlotKey, ComboSkill<_Id>>);

	struct _CombosContext<'ctx>(Ref<'ctx, _Combos>);

	impl BuildComboTreeLayout<_Id> for _CombosContext<'_> {
		fn build_combo_tree_layout(&self) -> ComboTreeLayout<SlotKey, ComboSkill<_Id>> {
			self.0.0.clone()
		}
	}

	impl ContextChanged for _CombosContext<'_> {
		fn context_changed(&self) -> bool {
			self.0.is_changed()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.add_systems(
			Update,
			_ComboOverview::update_from::<_PlayerParam, _Param, _Id>,
		);

		app
	}

	#[test]
	fn update_combos() {
		let mut app = setup();
		app.world_mut().spawn((
			_Player,
			_Combos(vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Symbol(Symbol::Line),
			]]),
		));
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view()
					.times(1)
					.with(eq(vec![vec![
						ComboTreeElement::Symbol(Symbol::Root),
						ComboTreeElement::Symbol(Symbol::Line),
					]]))
					.return_const(());
			}));

		app.update();
	}

	#[test]
	fn do_nothing_if_combos_was_not_added() {
		let mut app = setup();
		app.world_mut().spawn((_Player, _Combos(vec![])));
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().times(1).return_const(());
			}));

		app.update();
		app.update();
	}

	#[test]
	fn update_combos_again_after_combos_mut_deref() {
		let mut app = setup();
		let agent = app.world_mut().spawn((_Player, _Combos(vec![]))).id();
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().times(2).return_const(());
			}));

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Combos>()
			.as_deref_mut();
		app.update();
	}

	#[test]
	fn update_combos_after_combos_overview_added() {
		let mut app = setup();
		app.world_mut().spawn((_Player, _Combos(vec![])));

		app.update();
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().times(1).return_const(());
			}));
		app.update();
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		let mut app = setup();
		app.world_mut().spawn(_Combos(vec![]));
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().never();
			}));

		app.update();
	}
}
