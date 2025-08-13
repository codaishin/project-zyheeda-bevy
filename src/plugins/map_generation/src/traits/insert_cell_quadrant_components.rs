use common::zyheeda_commands::ZyheedaEntityCommands;
use std::collections::HashSet;

pub(crate) trait InsertCellQuadrantComponents {
	fn insert_cell_quadrant_components(
		&self,
		entity: &mut ZyheedaEntityCommands,
		different_quadrants: HashSet<Quadrant>,
	);
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Quadrant {
	Left,
	Forward,
	Diagonal,
}

pub(crate) trait PatternMatches {
	fn matches<const N: usize>(&self, pattern: [Quadrant; N]) -> bool;
}

impl PatternMatches for HashSet<Quadrant> {
	fn matches<const N: usize>(&self, pattern: [Quadrant; N]) -> bool {
		self == &HashSet::from(pattern)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn match_single_item() {
		let a = HashSet::from([Quadrant::Left]);

		assert!(a.matches([Quadrant::Left]));
	}

	#[test]
	fn match_single_item_multiple_items() {
		let a = HashSet::from([Quadrant::Left, Quadrant::Forward]);

		assert!(a.matches([Quadrant::Left, Quadrant::Forward]));
	}

	#[test]
	fn match_fails_when_object_larger() {
		let a = HashSet::from([Quadrant::Left, Quadrant::Forward]);

		assert!(!a.matches([Quadrant::Left]));
	}

	#[test]
	fn match_fails_when_object_smaller() {
		let a = HashSet::from([Quadrant::Left]);

		assert!(!a.matches([Quadrant::Left, Quadrant::Forward]));
	}
}
