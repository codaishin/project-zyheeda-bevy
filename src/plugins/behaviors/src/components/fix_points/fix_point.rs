use crate::components::fix_points::AnchorFixPointKey;
use bevy::prelude::*;
use common::{
	tools::{Index, bone::Bone},
	traits::{
		iteration::IterFinite,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct FixPoint<T>(pub(crate) T);

impl<T> FixPoint<T>
where
	T: IterFinite + ThreadSafe + Clone + Into<Bone>,
{
	pub(crate) fn insert(mut commands: Commands, names: Query<(Entity, &Name), Changed<Name>>) {
		let items = T::iterator().collect::<Vec<_>>();

		for (entity, name) in &names {
			let spawner = items.iter().find(|item| {
				let Bone(bone) = (*(*item)).into();
				bone == name.as_str()
			});

			match spawner {
				Some(spawner) => {
					commands.try_insert_on(entity, FixPoint(*spawner));
				}
				None => {
					commands.try_remove_from::<FixPoint<T>>(entity);
				}
			}
		}
	}
}

impl<T> From<FixPoint<T>> for AnchorFixPointKey
where
	T: Into<Index<usize>> + 'static,
{
	fn from(FixPoint(spawner): FixPoint<T>) -> Self {
		let Index(index) = spawner.into();
		AnchorFixPointKey::new::<FixPoint<T>>(index)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::iteration::{Iter, IterFinite};
	use std::any::TypeId;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _T {
		A,
		B,
	}

	impl IterFinite for _T {
		fn iterator() -> Iter<Self> {
			Iter(Some(_T::A))
		}

		fn next(Iter(current): &Iter<Self>) -> Option<Self> {
			match current.as_ref()? {
				_T::A => Some(_T::B),
				_T::B => None,
			}
		}
	}

	impl From<_T> for Bone {
		fn from(value: _T) -> Self {
			match value {
				_T::A => Bone("a"),
				_T::B => Bone("b"),
			}
		}
	}

	impl From<_T> for Index<usize> {
		fn from(value: _T) -> Self {
			match value {
				_T::A => Index(128),
				_T::B => Index(255),
			}
		}
	}

	fn setup<T>() -> App
	where
		for<'a> T: IterFinite + Into<Bone> + ThreadSafe,
	{
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, FixPoint::<T>::insert);

		app
	}

	#[test_case("invalid", None; "none")]
	#[test_case("a", Some(&FixPoint(_T::A)); "a")]
	#[test_case("b", Some(&FixPoint(_T::B)); "b")]
	fn insert(name: &str, expected: Option<&FixPoint<_T>>) {
		let mut app = setup::<_T>();
		let entity = app.world_mut().spawn(Name::from(name)).id();

		app.update();

		assert_eq!(expected, app.world().entity(entity).get::<FixPoint<_T>>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup::<_T>();
		let entity = app.world_mut().spawn(Name::from("a")).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<FixPoint<_T>>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<FixPoint<_T>>());
	}

	#[test]
	fn act_again_if_name_changed() {
		let mut app = setup::<_T>();
		let entity = app.world_mut().spawn(Name::from("a")).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<FixPoint<_T>>()
			.get_mut::<Name>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&FixPoint(_T::A)),
			app.world().entity(entity).get::<FixPoint::<_T>>()
		);
	}

	#[test]
	fn remove_fix_point_when_name_becomes_invalid() {
		let mut app = setup::<_T>();
		let entity = app.world_mut().spawn(Name::from("a")).id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(Name::from("unicorn"));
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<FixPoint<_T>>());
	}

	#[test]
	fn spawner_to_anchor_fix_point_key_has_correct_source() {
		assert!(
			[FixPoint(_T::A), FixPoint(_T::B)]
				.into_iter()
				.map(AnchorFixPointKey::from)
				.all(|key| key.source_type == TypeId::of::<FixPoint<_T>>())
		);
	}
}
