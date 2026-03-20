use super::{AddPrefabObserver, Prefab};
use crate::{
	systems::log::OnError,
	traits::prefab::Reapply::{Always, Never},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};

impl AddPrefabObserver for App {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self) -> &mut Self
	where
		TPrefab: Prefab<TDependencies> + Component,
		TDependencies: 'static,
	{
		match TPrefab::REAPPLY {
			Always => {
				self.add_observer(instantiate::<Insert, TPrefab, TDependencies>.pipe(OnError::log))
			}
			Never => {
				self.add_observer(instantiate::<Add, TPrefab, TDependencies>.pipe(OnError::log))
			}
		}
	}
}

fn instantiate<TEvent, TPrefab, TDependencies>(
	trigger: On<TEvent, TPrefab>,
	components: Query<&TPrefab>,
	mut commands: Commands,
	system_param: StaticSystemParam<TPrefab::TSystemParam<'_, '_>>,
) -> Result<(), TPrefab::TError>
where
	TEvent: EntityEvent,
	TPrefab: Prefab<TDependencies> + Component,
{
	let entity = trigger.event_target();
	let Ok(component) = components.get(entity) else {
		return Ok(());
	};
	let Ok(mut entity) = commands.get_entity(entity) else {
		return Ok(());
	};

	component.insert_prefab_components(&mut entity, system_param)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		errors::{ErrorData, Level},
		traits::prefab::PrefabEntityCommands,
	};
	use std::fmt::Display;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	#[derive(Component)]
	#[component(immutable)]
	struct _Component {
		prefab: Result<fn(usize) -> _Prefab, _Error>,
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Prefab(usize);

	struct _Dependency;

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Counter(usize);

	impl Prefab<_Dependency> for _Component {
		type TError = _Error;
		type TSystemParam<'w, 's> = ResMut<'w, _Counter>;

		fn insert_prefab_components(
			&self,
			entity: &mut impl PrefabEntityCommands,
			mut counter: StaticSystemParam<ResMut<_Counter>>,
		) -> Result<(), _Error> {
			match &self.prefab {
				Ok(prefab) => {
					counter.0 += 1;
					entity.try_insert(prefab(counter.0))
				}
				Err(error) => return Err(*error),
			};

			Ok(())
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), _Error>);

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Error;

	impl Display for _Error {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "_ERROR")
		}
	}

	impl ErrorData for _Error {
		fn level(&self) -> Level {
			Level::Error
		}

		fn label() -> impl Display {
			"_ERROR"
		}

		fn into_details(self) -> impl Display {
			self
		}
	}

	fn save_result(In(result): In<Result<(), _Error>>, mut commands: Commands) {
		commands.insert_resource(_Result(result));
	}

	fn setup<TEvent>() -> App
	where
		TEvent: EntityEvent,
	{
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Counter>();
		app.add_observer(instantiate::<TEvent, _Component, _Dependency>.pipe(save_result));

		app
	}

	#[test]
	fn call_prefab_instantiation_method() {
		let mut app = setup::<Insert>();

		let entity = app.world_mut().spawn(_Component {
			prefab: Ok(_Prefab),
		});

		assert_eq!(Some(&_Prefab(1)), entity.get::<_Prefab>());
	}

	#[test]
	fn return_error() {
		let mut app = setup::<Insert>();

		let entity = app.world_mut().spawn(_Component {
			prefab: Err(_Error),
		});

		assert_eq!(
			Some(&_Result(Err(_Error))),
			entity.get_resource::<_Result>(),
		);
	}

	const FAKE_ENTITY: Entity = match Entity::from_raw_u32(4242) {
		Some(e) => e,
		None => panic!("INVALID ENTITY"),
	};

	#[test_case(Add { entity: FAKE_ENTITY }, 1; "on add")]
	#[test_case(Insert { entity: FAKE_ENTITY }, 2; "on insert")]
	fn use_trigger_event<TEvent>(_: TEvent, expected_count: usize)
	where
		TEvent: EntityEvent,
	{
		let mut app = setup::<TEvent>();

		app.world_mut()
			.spawn(_Component {
				prefab: Ok(_Prefab),
			})
			.insert(_Component {
				prefab: Ok(_Prefab),
			});

		assert_eq!(
			&_Counter(expected_count),
			app.world().resource::<_Counter>(),
		);
	}
}
