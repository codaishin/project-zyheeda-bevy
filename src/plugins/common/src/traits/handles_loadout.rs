use crate::{
	tools::{
		action_key::slot::SlotKey,
		inventory_key::InventoryKey,
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::{GetParamEntry, RefInto},
		handles_localization::Token,
		thread_safe::ThreadSafe,
	},
};
use bevy::{ecs::component::Mutable, prelude::*};
use std::collections::HashSet;

pub trait HandlesLoadout {
	type TItemEntry: for<'a> RefInto<'a, Option<ItemToken<'a>>>
		+ for<'a> RefInto<'a, Option<SkillToken<'a>>>
		+ for<'a> RefInto<'a, Option<SkillIcon<'a>>>
		+ for<'a> RefInto<'a, Option<&'a SkillExecution>>;
	type TSkill: PartialEq
		+ Clone
		+ ThreadSafe
		+ for<'a> RefInto<'a, SkillToken<'a>>
		+ for<'a> RefInto<'a, SkillIcon<'a>>;
	type TSkills: IntoIterator<Item = Self::TSkill>;

	type TInventory: Component<Mutability = Mutable>
		+ ContainerKey<TKey = InventoryKey>
		+ ContainerItem<TItem = Self::TItemEntry>
		+ SwapInternal
		+ SwapExternal<Self::TSlots>
		+ for<'w, 's> GetParamEntry<'w, 's, InventoryKey, TEntry = Self::TItemEntry>;
	type TSlots: Component<Mutability = Mutable>
		+ ContainerKey<TKey = SlotKey>
		+ ContainerItem<TItem = Self::TItemEntry>
		+ SwapInternal
		+ SwapExternal<Self::TInventory>
		+ for<'w, 's> GetParamEntry<'w, 's, SlotKey, TEntry = Self::TItemEntry>
		+ for<'w, 's> GetParamEntry<'w, 's, AvailableSkills<SlotKey>, TEntry = Self::TSkills>;
	type TCombos: Component<Mutability = Mutable>
		+ ContainerKey<TKey = SlotKey>
		+ ContainerItem<TItem = Self::TSkill>
		+ for<'a> GetCombosOrdered
		+ for<'a> UpdateCombos
		+ NextConfiguredKeys<SlotKey>;
}

pub type Combo<TKey, TSkill> = Vec<(Vec<TKey>, TSkill)>;

pub trait NextConfiguredKeys<TKey> {
	fn next_keys(&self, combo_keys: &[TKey]) -> HashSet<TKey>;
}

pub trait GetCombosOrdered: ContainerKey + ContainerItem {
	/// Get combos with a consistent ordering.
	/// The specific ordering heuristic is up to the implementor.
	fn combos_ordered(&self) -> Vec<Combo<Self::TKey, Self::TItem>>;
}

pub trait ContainerKey {
	type TKey: Copy + ThreadSafe;
}

pub trait ContainerItem {
	type TItem;
}

pub trait SwapInternal: ContainerKey {
	fn swap_internal<TKey>(&mut self, a: TKey, b: TKey)
	where
		TKey: Into<Self::TKey> + 'static;
}

pub trait SwapExternal<TOther>: ContainerKey
where
	TOther: ContainerKey,
{
	fn swap_external<TKey, TOtherKey>(&mut self, other: &mut TOther, a: TKey, b: TOtherKey)
	where
		TKey: Into<Self::TKey> + 'static,
		TOtherKey: Into<TOther::TKey> + 'static;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AvailableSkills<T>(pub T);

pub trait UpdateCombos: ContainerKey + ContainerItem {
	// FIXME: return indication of success?
	fn update_combos(&mut self, combos: Combo<Self::TKey, Option<Self::TItem>>);
}

#[derive(Debug, PartialEq, Clone)]
pub struct ItemToken<'a>(pub &'a Token);

#[derive(Debug, PartialEq, Clone)]
pub struct SkillToken<'a>(pub &'a Token);

#[derive(Debug, PartialEq, Clone)]
pub struct SkillIcon<'a>(pub &'a Handle<Image>);

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
