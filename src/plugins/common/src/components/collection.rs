use crate::{
	errors::Error,
	traits::{
		try_from_resource::TryFromResource,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct Collection<TElement>(pub Vec<TElement>);

#[derive(Debug, PartialEq)]
pub struct MovedCollections<TElement> {
	collections: Vec<(Entity, Collection<TElement>)>,
	errors: Vec<Result<(), Error>>,
}

impl<TElement> Collection<TElement> {
	pub fn new<const N: usize>(elements: [TElement; N]) -> Self {
		Self(Vec::from(elements))
	}

	pub fn move_out(
		mut commands: Commands,
		collections: Query<(Entity, &Collection<TElement>)>,
	) -> MovedCollections<TElement>
	where
		TElement: Clone + Sync + Send + 'static,
	{
		MovedCollections {
			collections: collections
				.iter()
				.map(|(entity, collection)| {
					commands.try_remove_from::<Collection<TElement>>(entity);
					(entity, collection.clone())
				})
				.collect(),
			errors: vec![],
		}
	}

	pub fn insert_as<TAs>(
		In(collections): In<MovedCollections<TElement>>,
		mut commands: Commands,
	) -> Vec<Result<(), Error>>
	where
		TAs: Component + From<Collection<TElement>>,
	{
		for (entity, collection) in collections.collections {
			commands.try_insert_on(entity, TAs::from(collection));
		}

		collections.errors
	}

	pub fn map_elements_from<TFrom>(
		In(moved): In<MovedCollections<TFrom>>,
		resource: Res<TElement::TResource>,
	) -> MovedCollections<TElement>
	where
		TFrom: Sync + Send + 'static,
		TElement: TryFromResource<TFrom>,
		TElement::TError: Into<Error>,
	{
		let mut errors = moved.errors;
		let collections = moved
			.collections
			.into_iter()
			.map(map_collections(resource.as_ref(), &mut errors))
			.collect();

		MovedCollections {
			collections,
			errors,
		}
	}
}

fn map_collections<'a, TElement, TFrom>(
	resource: &'a <TElement as TryFromResource<TFrom>>::TResource,
	errors: &'a mut Vec<Result<(), Error>>,
) -> impl FnMut((Entity, Collection<TFrom>)) -> (Entity, Collection<TElement>) + 'a
where
	TFrom: Sync + Send + 'static,
	TElement: TryFromResource<TFrom>,
	TElement::TError: Into<Error>,
{
	move |(entity, Collection(elements)): (Entity, Collection<TFrom>)| {
		(
			entity,
			Collection(
				elements
					.into_iter()
					.filter_map(map_elements(resource, errors))
					.collect(),
			),
		)
	}
}

fn map_elements<'a, TElement, TFrom>(
	resource: &'a <TElement as TryFromResource<TFrom>>::TResource,
	errors: &'a mut Vec<Result<(), Error>>,
) -> impl FnMut(TFrom) -> Option<TElement> + 'a
where
	TElement: TryFromResource<TFrom>,
	TElement::TError: Into<Error>,
{
	|element| match TElement::try_from_resource(element, resource) {
		Ok(element) => Some(element),
		Err(error) => {
			errors.push(Err(error.into()));
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::collection::MovedCollections, errors::Level};
	use bevy::ecs::system::RunSystemOnce;

	#[derive(Debug, PartialEq, Clone)]
	struct _Mappable(bool);

	#[derive(Debug, PartialEq, Clone)]
	struct _Elem;

	#[derive(Debug, PartialEq)]
	struct _Error;

	#[derive(Resource, Default)]
	struct _Resource;

	impl TryFromResource<_Mappable> for _Elem {
		type TResource = _Resource;
		type TError = _Error;

		fn try_from_resource(
			_Mappable(mappable): _Mappable,
			_: &Self::TResource,
		) -> Result<Self, Self::TError> {
			if !mappable {
				return Err(_Error);
			}

			Ok(_Elem)
		}
	}

	impl From<_Error> for Error {
		fn from(_: _Error) -> Self {
			Error {
				msg: "failed hard".to_owned(),
				lvl: Level::Error,
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.init_resource::<_Resource>();

		app
	}

	#[test]
	fn move_out_returns_collection() {
		let mut app = setup();
		let entities = [
			app.world_mut().spawn(Collection::new([_Elem, _Elem])).id(),
			app.world_mut().spawn(Collection::new([_Elem])).id(),
		];

		let moved_out = app
			.world_mut()
			.run_system_once(Collection::<_Elem>::move_out);

		assert_eq!(
			MovedCollections {
				collections: vec![
					(entities[0], Collection::new([_Elem, _Elem])),
					(entities[1], Collection::new([_Elem])),
				],
				errors: vec![]
			},
			moved_out,
		);
	}

	#[test]
	fn move_out_removes_collection() {
		let mut app = setup();
		let entities = [
			app.world_mut().spawn(Collection::new([_Elem, _Elem])).id(),
			app.world_mut().spawn(Collection::new([_Elem])).id(),
		];

		app.world_mut()
			.run_system_once(Collection::<_Elem>::move_out);

		assert_eq!(
			(None, None),
			(
				app.world().entity(entities[0]).get::<Collection<_Elem>>(),
				app.world().entity(entities[1]).get::<Collection<_Elem>>(),
			),
		);
	}

	#[test]
	fn insert_collection() {
		#[derive(Component, Debug, PartialEq)]
		struct _MyCollection(Collection<_Elem>);

		impl From<Collection<_Elem>> for _MyCollection {
			fn from(value: Collection<_Elem>) -> Self {
				Self(value)
			}
		}

		let mut app = setup();
		let entities = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		let moved = MovedCollections {
			collections: vec![
				(entities[0], Collection::new([_Elem, _Elem])),
				(entities[1], Collection::new([_Elem])),
			],
			errors: vec![],
		};

		app.world_mut()
			.run_system_once_with(moved, Collection::<_Elem>::insert_as::<_MyCollection>);

		assert_eq!(
			(
				Some(&_MyCollection(Collection::new([_Elem, _Elem]))),
				Some(&_MyCollection(Collection::new([_Elem])))
			),
			(
				app.world().entity(entities[0]).get::<_MyCollection>(),
				app.world().entity(entities[1]).get::<_MyCollection>(),
			),
		);
	}

	#[test]
	fn return_errors_when_inserting() {
		let mut app = setup();
		let entities = [
			app.world_mut().spawn_empty().id(),
			app.world_mut().spawn_empty().id(),
		];
		let moved = MovedCollections {
			collections: vec![
				(entities[0], Collection::new([_Elem, _Elem])),
				(entities[1], Collection::new([_Elem])),
			],
			errors: vec![Err(Error {
				msg: "Such a shame".to_owned(),
				lvl: Level::Error,
			})],
		};

		let errors = app
			.world_mut()
			.run_system_once_with(moved, Collection::<_Elem>::insert_as::<Collection<_Elem>>);

		assert_eq!(
			vec![Err(Error {
				msg: "Such a shame".to_owned(),
				lvl: Level::Error,
			})],
			errors,
		);
	}

	#[test]
	fn map_elements() {
		let mut app = setup();
		let moved = MovedCollections {
			collections: vec![
				(
					Entity::from_raw(42),
					Collection::new([_Mappable(true), _Mappable(false)]),
				),
				(Entity::from_raw(66), Collection::new([_Mappable(true)])),
			],
			errors: vec![],
		};

		let result = app
			.world_mut()
			.run_system_once_with(moved, Collection::<_Elem>::map_elements_from::<_Mappable>);

		assert_eq!(
			vec![
				(Entity::from_raw(42), Collection::new([_Elem,]),),
				(Entity::from_raw(66), Collection::new([_Elem]),),
			],
			result.collections,
		);
	}

	#[test]
	fn map_errors() {
		let mut app = setup();
		let moved = MovedCollections {
			collections: vec![
				(
					Entity::from_raw(42),
					Collection::new([_Mappable(true), _Mappable(false)]),
				),
				(Entity::from_raw(66), Collection::new([_Mappable(true)])),
			],
			errors: vec![],
		};

		let result = app
			.world_mut()
			.run_system_once_with(moved, Collection::<_Elem>::map_elements_from::<_Mappable>);

		assert_eq!(
			vec![Err(Error {
				msg: "failed hard".to_owned(),
				lvl: Level::Error,
			})],
			result.errors,
		);
	}

	#[test]
	fn map_elements_and_expand_errors() {
		let mut app = setup();
		let moved = MovedCollections {
			collections: vec![(Entity::from_raw(42), Collection::new([_Mappable(false)]))],
			errors: vec![Err(Error {
				msg: "such a previous shame".to_owned(),
				lvl: Level::Error,
			})],
		};

		let result = app
			.world_mut()
			.run_system_once_with(moved, Collection::<_Elem>::map_elements_from::<_Mappable>);

		assert_eq!(
			vec![
				Err(Error {
					msg: "such a previous shame".to_owned(),
					lvl: Level::Error,
				}),
				Err(Error {
					msg: "failed hard".to_owned(),
					lvl: Level::Error,
				})
			],
			result.errors,
		);
	}

	#[test]
	fn integration_of_moving_out_mapping_and_reinserting() {
		/* For documentation purposes
		 * The `result` would usually be collected by a logging system
		 */

		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Collection::new([_Mappable(true), _Mappable(false)]))
			.id();
		let systems = Collection::<_Mappable>::move_out
			.pipe(Collection::<_Elem>::map_elements_from::<_Mappable>)
			.pipe(Collection::<_Elem>::insert_as::<Collection<_Elem>>);

		let result = app.world_mut().run_system_once(systems);

		assert_eq!(
			(
				Some(&Collection::new([_Elem])),
				vec![Err(Error {
					msg: "failed hard".to_owned(),
					lvl: Level::Error,
				})]
			),
			(
				app.world().entity(entity).get::<Collection<_Elem>>(),
				result
			),
		);
	}
}
