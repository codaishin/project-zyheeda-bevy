use crate::traits::{UpdateCombosView, build_combo_tree_layout::BuildComboTreeLayout};
use bevy::{ecs::component::Mutable, prelude::*};
use common::traits::thread_safe::ThreadSafe;

impl<T> UpdateComboOverview for T where T: Component<Mutability = Mutable> {}

pub(crate) trait UpdateComboOverview: Component<Mutability = Mutable> + Sized {
	fn update_combos_overview<TSkill, TLayoutBuilder>(
		layout_builder: Res<TLayoutBuilder>,
		mut combo_overviews: Query<&mut Self>,
	) where
		Self: UpdateCombosView<TSkill>,
		TSkill: ThreadSafe,
		TLayoutBuilder: Resource + BuildComboTreeLayout<TSkill>,
	{
		let Ok(mut combo_overview) = combo_overviews.single_mut() else {
			return;
		};

		if !layout_builder.is_changed() && !combo_overview.is_added() {
			return;
		}

		combo_overview.update_combos_view(layout_builder.build_combo_tree_layout());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::build_combo_tree_layout::{ComboTreeElement, ComboTreeLayout, Symbol};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::ops::DerefMut;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Clone)]
	struct _Skill;

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

	#[derive(Resource, Clone)]
	struct _Combos(ComboTreeLayout<_Skill>);

	impl BuildComboTreeLayout<_Skill> for _Combos {
		fn build_combo_tree_layout(&self) -> ComboTreeLayout<_Skill> {
			self.0.clone()
		}
	}

	fn setup(combos: _Combos) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(combos);
		app.add_systems(
			Update,
			_ComboOverview::update_combos_overview::<_Skill, _Combos>,
		);

		app
	}

	#[test]
	fn update_combos() {
		let mut app = setup(_Combos(vec![vec![
			ComboTreeElement::Symbol(Symbol::Root),
			ComboTreeElement::Symbol(Symbol::Line),
		]]));
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
		let mut app = setup(_Combos(vec![vec![
			ComboTreeElement::Symbol(Symbol::Root),
			ComboTreeElement::Symbol(Symbol::Line),
		]]));
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().times(1).return_const(());
			}));

		app.update();
		app.update();
	}

	#[test]
	fn update_combos_again_after_combos_mut_deref() {
		let mut app = setup(_Combos(vec![vec![
			ComboTreeElement::Symbol(Symbol::Root),
			ComboTreeElement::Symbol(Symbol::Line),
		]]));
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().times(2).return_const(());
			}));

		app.update();
		app.world_mut().resource_mut::<_Combos>().deref_mut();
		app.update();
	}

	#[test]
	fn update_combos_after_combos_overview_added() {
		let mut app = setup(_Combos(vec![vec![
			ComboTreeElement::Symbol(Symbol::Root),
			ComboTreeElement::Symbol(Symbol::Line),
		]]));

		app.update();
		app.world_mut()
			.spawn(_ComboOverview::new().with_mock(|mock| {
				mock.expect_update_combos_view().times(1).return_const(());
			}));
		app.update();
	}
}
