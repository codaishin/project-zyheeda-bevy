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
				self.add_observer(instantiate::<Insert<TPrefab>, TDependencies>.pipe(OnError::log))
			}
			Never => {
				self.add_observer(instantiate::<Add<TPrefab>, TDependencies>.pipe(OnError::log))
			}
		}
	}
}

fn instantiate<TPattern, TDependencies>(
	trigger: On<TPattern>,
	components: Query<&TPattern::Components>,
	mut commands: Commands,
	system_param: StaticSystemParam<<TPattern::Components as Prefab<TDependencies>>::TSystemParam>,
) -> Result<(), <TPattern::Components as Prefab<TDependencies>>::TError>
where
	TPattern: EventPattern<Event: EntityEvent>,
	TPattern::Components: Prefab<TDependencies> + Component,
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
	use std::{fmt::Display, marker::PhantomData};
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
		type TSystemParam = ResMut<'static, _Counter>;

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

	fn setup<TPattern>() -> App
	where
		TPattern: EventPattern<Event: EntityEvent, Components = _Component>,
	{
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_Counter>();
		app.add_observer(instantiate::<TPattern, _Dependency>.pipe(save_result));

		app
	}

	#[test]
	fn call_prefab_instantiation_method() {
		let mut app = setup::<Insert<_Component>>();

		let entity = app.world_mut().spawn(_Component {
			prefab: Ok(_Prefab),
		});

		assert_eq!(Some(&_Prefab(1)), entity.get::<_Prefab>());
	}

	#[test]
	fn return_error() {
		let mut app = setup::<Insert<_Component>>();

		let entity = app.world_mut().spawn(_Component {
			prefab: Err(_Error),
		});

		assert_eq!(
			Some(&_Result(Err(_Error))),
			entity.get_resource::<_Result>(),
		);
	}

	struct _T<T>(PhantomData<T>);

	impl<T> _T<T> {
		fn mark() -> Self {
			Self(PhantomData)
		}
	}

	#[test_case(_T::<Add::<_Component>>::mark, 1; "on add")]
	#[test_case(_T::<Insert::<_Component>>::mark, 2; "on insert")]
	fn use_trigger_event<TPattern>(_: fn() -> _T<TPattern>, expected_count: usize)
	where
		TPattern: EventPattern<Event: EntityEvent, Components = _Component>,
	{
		let mut app = setup::<TPattern>();

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
