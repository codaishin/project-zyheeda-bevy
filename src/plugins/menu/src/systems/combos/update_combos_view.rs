use crate::traits::{build_combo_tree_layout::BuildComboTreeLayout, UpdateCombosView};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

impl<T, TLayoutBuilder> UpdateComboOverview<TLayoutBuilder> for T {}

pub(crate) trait UpdateComboOverview<TLayoutBuilder> {
	fn update_combos_overview<TAgent, TCombos, TSkill>(
		get_layout_builder: impl Fn(&TCombos) -> TLayoutBuilder,
	) -> impl Fn(Query<&TCombos, With<TAgent>>, Query<&mut Self>)
	where
		Self: Component + UpdateCombosView<TSkill> + Sized,
		TAgent: Component,
		TCombos: Component,
		TSkill: ThreadSafe,
		TLayoutBuilder: BuildComboTreeLayout<TSkill>,
	{
		move |combos: Query<&TCombos, With<TAgent>>, mut combo_overviews: Query<&mut Self>| {
			let Ok(layout_builder) = combos.get_single().map(&get_layout_builder) else {
				return;
			};

			let Ok(mut combo_overview) = combo_overviews.get_single_mut() else {
				return;
			};

			combo_overview.update_combos_view(layout_builder.build_combo_tree_layout());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::build_combo_tree_layout::{ComboTreeElement, ComboTreeLayout, Symbol};
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Debug, PartialEq, Clone)]
	struct _Skill;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks, Debug)]
	struct _ComboOverview {
		mock: Mock_ComboOverview,
	}

	#[automock]
	impl UpdateCombosView<_Skill> for _ComboOverview {
		fn update_combos_view(&mut self, combos: ComboTreeLayout<_Skill>) {
			self.mock.update_combos_view(combos)
		}
	}

	#[derive(Component, Clone)]
	struct _Combos(ComboTreeLayout<_Skill>);

	impl BuildComboTreeLayout<_Skill> for _Combos {
		fn build_combo_tree_layout(self) -> ComboTreeLayout<_Skill> {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			_ComboOverview::update_combos_overview::<_Agent, _Combos, _Skill>(_Combos::clone),
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
