use crate::{
	components::persistent_entity::PersistentEntity,
	traits::{
		accessors::get::ViewField,
		iteration::{Iter, IterFinite},
	},
};
use bevy::ecs::{query::QueryFilter, system::SystemParam};
use serde::{Deserialize, Serialize};

pub trait HandlesEnemies {
	type TEnemy<TFilter>: for<'w, 's> SystemParam<
		Item<'w, 's>: IntoIterator<Item = PersistentEntity>,
	>
	where
		TFilter: QueryFilter + 'static;
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyType {
	VoidSphere,
}

impl IterFinite for EnemyType {
	fn iterator() -> Iter<Self> {
		Iter(Some(EnemyType::VoidSphere))
	}

	fn next(Iter(current): &Iter<Self>) -> Option<Self> {
		match current.as_ref()? {
			EnemyType::VoidSphere => None,
		}
	}
}

impl ViewField for EnemyType {
	type TValue<'a> = Self;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_enemy_types_finite() {
		assert_eq!(
			vec![EnemyType::VoidSphere],
			EnemyType::iterator().collect::<Vec<_>>()
		);
	}
}
