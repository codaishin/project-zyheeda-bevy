use crate::traits::{combo_tree_layout::GetComboTreeLayout, UpdateCombosView};
use bevy::prelude::{Component, Query, With};

pub(crate) fn update_combos_view<TAgent, TCombos, TComboOverview>(
	agents: Query<&TCombos, With<TAgent>>,
	mut combo_overviews: Query<&mut TComboOverview>,
) where
	TAgent: Component,
	TCombos: Component + GetComboTreeLayout,
	TComboOverview: Component + UpdateCombosView,
{
	let Ok(combos) = agents.get_single() else {
		return;
	};

	let Ok(mut combo_overview) = combo_overviews.get_single_mut() else {
		return;
	};

	combo_overview.update_combos_view(combos.combo_tree_layout());
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::combo_tree_layout::{ComboTreeElement, ComboTreeLayout, Symbol};
	use bevy::app::{App, Update};
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks, Debug)]
	struct _ComboOverview {
		mock: Mock_ComboOverview,
	}

	#[automock]
	impl UpdateCombosView for _ComboOverview {
		fn update_combos_view(&mut self, combos: ComboTreeLayout) {
			self.mock.update_combos_view(combos)
		}
	}

	#[derive(Component)]
	struct _Combos(ComboTreeLayout);

	impl GetComboTreeLayout for _Combos {
		fn combo_tree_layout(&self) -> ComboTreeLayout {
			self.0.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			update_combos_view::<_Agent, _Combos, _ComboOverview>,
		);

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
	fn do_nothing_if_agent_missing() {
		let mut app = setup();
		app.world_mut().spawn(_Combos(vec![vec![
			ComboTreeElement::Symbol(Symbol::Root),
			ComboTreeElement::Symbol(Symbol::Line),
		]]));
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().never().return_const(());
			}));

		app.update();
	}
}
