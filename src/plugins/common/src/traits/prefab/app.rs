use super::{AddPrefabObserver, Prefab};
use crate::{errors::Error, systems::log::log};
use bevy::prelude::*;

impl AddPrefabObserver for App {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self) -> &mut Self
	where
		TPrefab: Prefab<TDependencies>,
		TDependencies: 'static,
	{
		self.add_observer(instantiate_prefab::<TPrefab, TDependencies>.pipe(log))
	}
}

fn instantiate_prefab<TPrefab, TDependencies>(
	trigger: Trigger<OnAdd, TPrefab>,
	components: Query<&TPrefab>,
	mut commands: Commands,
) -> Result<(), Error>
where
	TPrefab: Prefab<TDependencies>,
{
	let entity = trigger.target();
	let Ok(component) = components.get(entity) else {
		return Ok(());
	};
	let Ok(mut entity) = commands.get_entity(entity) else {
		return Ok(());
	};

	component.insert_prefab_components(&mut entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		errors::{Error, Level},
		test_tools::utils::SingleThreadedApp,
	};

	#[derive(Component)]
	struct _Component(Result<_Prefab, Error>);

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	enum _Prefab {
		A,
		B,
	}

	struct _Dependency;

	impl Prefab<_Dependency> for _Component {
		fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
			match &self.0 {
				Ok(prefab) => entity.insert(*prefab),
				Err(error) => return Err(error.clone()),
			};

			Ok(())
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Error>);

	fn save_result(In(result): In<Result<(), Error>>, mut commands: Commands) {
		commands.insert_resource(_Result(result));
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(instantiate_prefab::<_Component, _Dependency>.pipe(save_result));

		app
	}

	#[test]
	fn call_prefab_instantiation_method() {
		let mut app = setup();

		let entity = app.world_mut().spawn(_Component(Ok(_Prefab::A)));

		assert_eq!(Some(&_Prefab::A), entity.get::<_Prefab>());
	}

	#[test]
	fn return_error() {
		let mut app = setup();
		let error = Error {
			msg: "my error".to_owned(),
			lvl: Level::Error,
		};

		let entity = app.world_mut().spawn(_Component(Err(error.clone())));

		assert_eq!(Some(&_Result(Err(error))), entity.get_resource::<_Result>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(_Component(Ok(_Prefab::A)));
		entity.insert(_Component(Ok(_Prefab::B)));

		assert_eq!(Some(&_Prefab::A), entity.get::<_Prefab>());
	}
}
