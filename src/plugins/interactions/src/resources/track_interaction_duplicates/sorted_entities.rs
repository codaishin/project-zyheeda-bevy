use bevy::prelude::Entity;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(super) struct SortedEntities([Entity; 2]);

impl From<[Entity; 2]> for SortedEntities {
	fn from(mut value: [Entity; 2]) -> Self {
		value.sort();

		SortedEntities(value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::Entity;

	#[test]
	fn sorted_entities_sorts() {
		let unsorted = [Entity::from_raw(42), Entity::from_raw(4)];

		let sorted = SortedEntities::from(unsorted);

		assert_eq!(
			SortedEntities([Entity::from_raw(4), Entity::from_raw(42),]),
			sorted
		)
	}
}
