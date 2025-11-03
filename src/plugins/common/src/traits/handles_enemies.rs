use crate::traits::{
	accessors::get::Property,
	iteration::{Iter, IterFinite},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait HandlesEnemies {
	type TEnemy: Component;
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

impl Property for EnemyType {
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
