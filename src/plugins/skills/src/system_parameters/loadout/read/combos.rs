use crate::{components::combos::Combos, system_parameters::loadout::LoadoutReader};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{ContextChanged, GetContext},
		handles_loadout::combos::{
			Combo,
			Combos as CombosMarker,
			GetCombosOrdered,
			NextConfiguredKeys,
		},
	},
};
use std::{collections::HashSet, ops::Deref};

impl GetContext<CombosMarker> for LoadoutReader<'_, '_> {
	type TContext<'ctx> = CombosView<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx LoadoutReader,
		CombosMarker { entity }: CombosMarker,
	) -> Option<Self::TContext<'ctx>> {
		let (_, _, combos, _) = param.agents.get(entity).ok()?;

		Some(CombosView { combos })
	}
}

#[derive(Debug)]
pub struct CombosView<'a> {
	combos: Ref<'a, Combos>,
}

impl PartialEq for CombosView<'_> {
	fn eq(&self, other: &Self) -> bool {
		self.combos.deref() == other.combos.deref()
	}
}

impl GetCombosOrdered for CombosView<'_> {
	type TSkill = <Combos as GetCombosOrdered>::TSkill;

	fn combos_ordered(&self) -> Vec<Combo<SlotKey, Self::TSkill>> {
		self.combos.combos_ordered()
	}
}

impl NextConfiguredKeys<SlotKey> for CombosView<'_> {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		self.combos.next_keys(combo_keys)
	}
}

impl ContextChanged for CombosView<'_> {
	fn context_changed(&self) -> bool {
		self.combos.is_changed()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			combo_node::ComboNode,
			combos::Combos,
			inventory::Inventory,
			queue::Queue,
			slots::Slots,
		},
		item::Item,
		skills::Skill,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{tools::action_key::slot::SlotKey, traits::handles_localization::Token};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<Item>>();
		app.init_resource::<Assets<Skill>>();

		app
	}

	#[test]
	fn slot_item() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Slots::default(),
				Inventory::default(),
				Combos::from(ComboNode::new([(
					SlotKey(42),
					(
						Skill {
							token: Token::from("my skill"),
							..default()
						},
						ComboNode::default(),
					),
				)])),
				Queue::default(),
			))
			.id();

		app.world_mut().run_system_once(
			move |loadout: LoadoutReader, combos: Query<Ref<Combos>>| {
				let ctx = LoadoutReader::get_context(&loadout, CombosMarker { entity });
				let combos = combos.get(entity).unwrap();

				assert_eq!(Some(CombosView { combos }), ctx);
			},
		)
	}
}
