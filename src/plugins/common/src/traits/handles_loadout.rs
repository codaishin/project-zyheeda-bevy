use crate::{
	tools::{
		action_key::slot::PlayerSlot,
		inventory_key::InventoryKey,
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::{GetFromParam, RefInto},
		handles_combo_menu::{Combo, GetCombosOrdered, NextConfiguredKeys},
		handles_localization::Token,
	},
};
use bevy::{ecs::component::Mutable, prelude::*};

pub trait HandlesLoadout {
	type TItem: for<'a> RefInto<'a, ItemToken<'a>>
		+ for<'a> RefInto<'a, SkillToken<'a>>
		+ for<'a> RefInto<'a, SkillIcon<'a>>
		+ for<'a> RefInto<'a, &'a SkillExecution>;
	type TSkill: PartialEq
		+ Clone
		+ for<'a> RefInto<'a, SkillToken<'a>>
		+ for<'a> RefInto<'a, SkillIcon<'a>>;
	type TSkills: IntoIterator<Item = Self::TSkill>;

	type TInventory: Component<Mutability = Mutable>
		+ SwapInternal<InventoryKey>
		+ SwapExternal<Self::TSlots, InventoryKey, PlayerSlot>
		+ for<'w, 's> GetFromParam<'w, 's, InventoryKey, TValue = Self::TItem>;
	type TSlots: Component<Mutability = Mutable>
		+ SwapInternal<PlayerSlot>
		+ SwapExternal<Self::TInventory, PlayerSlot, InventoryKey>
		+ for<'w, 's> GetFromParam<'w, 's, PlayerSlot, TValue = Self::TItem>
		+ for<'w, 's> GetFromParam<'w, 's, AvailableSkills<PlayerSlot>, TValue = Self::TSkills>;
	type TCombos: Component<Mutability = Mutable>
		+ for<'a> GetCombosOrdered<Self::TSkill, PlayerSlot>
		+ for<'a> UpdateCombos<Self::TSkill>
		+ NextConfiguredKeys<PlayerSlot>;
}

pub trait SwapInternal<TKey> {
	fn swap_internal(&mut self, a: TKey, b: TKey);
}

pub trait SwapExternal<TOther, TKey, TKeyOther> {
	fn swap_external(&mut self, other: &mut TOther, a: TKey, b: TKeyOther);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AvailableSkills<T>(pub T);

pub trait UpdateCombos<TSkill> {
	// FIXME: return indication of success?
	fn update_combos(&mut self, combos: Combo<PlayerSlot, Option<TSkill>>);
}

pub struct ItemToken<'a>(pub Option<&'a Token>);

pub struct SkillToken<'a>(pub Option<&'a Token>);

pub struct SkillIcon<'a>(pub Option<&'a Handle<Image>>);

/// Prove of concept of using generic system parameters
// FIXME: REMOVE IN FINAL PR
#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{StaticSystemParam, SystemParam};

	trait UtilizeSystemParam<'w, 's> {
		type TParam: SystemParam;

		fn utilize(&mut self, p: &Self::TParam);
	}

	#[derive(Component, Debug, PartialEq)]
	struct MyComponent {
		utilized: bool,
	}

	impl<'w, 's> UtilizeSystemParam<'w, 's> for MyComponent {
		type TParam = Commands<'w, 's>;

		fn utilize(&mut self, _: &Self::TParam) {
			self.utilized = true;
		}
	}

	fn generic_system<TParam>(t: StaticSystemParam<TParam>, mut c: Query<&mut MyComponent>)
	where
		TParam: SystemParam + 'static,
		for<'w, 's> MyComponent: UtilizeSystemParam<'w, 's, TParam = TParam::Item<'w, 's>>,
	{
		for mut c in &mut c {
			c.utilize(&t);
		}
	}

	/// Makes the used system param fully generic
	fn add_generic_system<TParam>(app: &mut App)
	where
		TParam: SystemParam + 'static,
		for<'w, 's> MyComponent: UtilizeSystemParam<'w, 's, TParam = TParam::Item<'w, 's>>,
	{
		app.add_systems(Update, generic_system::<TParam>);
	}

	#[test]
	fn bar() {
		let mut app = App::new();
		add_generic_system::<Commands>(&mut app);
		let entity = app.world_mut().spawn(MyComponent { utilized: false }).id();

		app.update();

		assert_eq!(
			Some(&MyComponent { utilized: true }),
			app.world().entity(entity).get::<MyComponent>(),
		);
	}
}
