use bevy::prelude::*;
use std::{any::TypeId, collections::HashSet};

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct Uniques {
	types: HashSet<TypeId>,
}

impl Uniques {
	pub(crate) fn mut_from(app: &mut App) -> Mut<'_, Self> {
		if app.world().get_resource::<Self>().is_none() {
			app.world_mut().init_resource::<Self>();
		}

		app.world_mut().resource_mut::<Self>()
	}

	pub(crate) fn register<T>(&mut self) -> IsUnique
	where
		T: 'static,
	{
		IsUnique(self.types.insert(TypeId::of::<T>()))
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct IsUnique(bool);

impl IsUnique {
	pub(crate) fn is_unique(self) -> bool {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup(unique: Option<Uniques>) -> App {
		let mut app = App::new().single_threaded(Update);

		if let Some(unique) = unique {
			app.insert_resource(unique);
		}

		app
	}

	#[test]
	fn get_from_app() {
		let mut app = setup(Some(Uniques {
			types: HashSet::from([TypeId::of::<u32>()]),
		}));

		assert_eq!(
			Uniques {
				types: HashSet::from([TypeId::of::<u32>()]),
			},
			*Uniques::mut_from(&mut app),
		);
	}

	#[test]
	fn get_new_in_app() {
		let mut app = setup(None);

		assert_eq!(Uniques::default(), *Uniques::mut_from(&mut app),);
	}

	#[test]
	fn register_unique() {
		let mut unique = Uniques::default();

		assert!(unique.register::<u32>().is_unique());
	}

	#[test]
	fn register_not_unique() {
		let mut unique = Uniques::default();
		unique.register::<u32>();

		assert!(!unique.register::<u32>().is_unique());
	}
}
