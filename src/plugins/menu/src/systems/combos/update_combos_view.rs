use crate::traits::{UpdateCombosView, build_combo_tree_layout::BuildComboTreeLayout};
use bevy::{ecs::component::Mutable, prelude::*};

impl<T> UpdateComboOverview for T where T: Component<Mutability = Mutable> + UpdateCombosView {}

pub(crate) trait UpdateComboOverview:
	Component<Mutability = Mutable> + UpdateCombosView + Sized
{
	fn update_from<TAgent, TCombos>(
		mut combo_overviews: Query<&mut Self>,
		combos: Query<Ref<TCombos>, With<TAgent>>,
	) where
		TAgent: Component,
		TCombos: Component + BuildComboTreeLayout<TKey = Self::TKey, TItem = Self::TItem>,
	{
		for combo in &combos {
			for mut combo_overview in &mut combo_overviews {
				if !combo.is_changed() && !combo_overview.is_added() {
					continue;
				}

				combo_overview.update_combos_view(combo.build_combo_tree_layout());
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::build_combo_tree_layout::{ComboTreeElement, ComboTreeLayout, Symbol};
	use common::traits::handles_loadout::loadout::{LoadoutItem, LoadoutKey};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Clone, Copy)]
	pub struct _Key;

	#[derive(Debug, PartialEq, Clone)]
	pub struct _Skill;

	#[derive(Component, NestedMocks, Debug)]
	struct _ComboOverview {
		mock: Mock_ComboOverview,
	}

	impl LoadoutKey for _ComboOverview {
		type TKey = _Key;
	}

	impl LoadoutItem for _ComboOverview {
		type TItem = _Skill;
	}

	impl LoadoutKey for Mock_ComboOverview {
		type TKey = _Key;
	}

	impl LoadoutItem for Mock_ComboOverview {
		type TItem = _Skill;
	}

	#[automock]
	impl UpdateCombosView for _ComboOverview {
		fn update_combos_view(&mut self, combos: ComboTreeLayout<_Key, _Skill>) {
			self.mock.update_combos_view(combos)
		}
	}

	#[derive(Component, Clone)]
	struct _Combos(ComboTreeLayout<_Key, _Skill>);

	impl LoadoutKey for _Combos {
		type TKey = _Key;
	}

	impl LoadoutItem for _Combos {
		type TItem = _Skill;
	}

	impl BuildComboTreeLayout for _Combos {
		fn build_combo_tree_layout(&self) -> ComboTreeLayout<_Key, _Skill> {
			self.0.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _ComboOverview::update_from::<_Agent, _Combos>);

		app
	}

	#[test]
	fn update_combos() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
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
		app.world_mut().spawn((_Agent, _Combos(vec![])));
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
		let agent = app.world_mut().spawn((_Agent, _Combos(vec![]))).id();
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
		app.world_mut().spawn((_Agent, _Combos(vec![])));

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
