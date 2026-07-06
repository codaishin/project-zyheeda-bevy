use crate::components::bar::Bar;
use bevy::ecs::system::{StaticSystemParam, SystemParam};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};

impl Bar {
	pub(crate) fn add_to<TAgents>(agents: StaticSystemParam<TAgents>, mut commands: ZyheedaCommands)
	where
		TAgents: for<'w, 's> SystemParam<Item<'w, 's>: IntoIterator<Item = PersistentEntity>>,
	{
		let agents = agents.into_inner();

		for entity in agents {
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert_if_new(Bar::default());
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::*;
	use common::CommonPlugin;
	use std::vec::IntoIter;
	use testing::SingleThreadedApp;

	#[derive(SystemParam)]
	struct _AgentsParam<'w> {
		agents: Res<'w, _Agents>,
	}

	impl IntoIterator for _AgentsParam<'_> {
		type Item = PersistentEntity;
		type IntoIter = IntoIter<PersistentEntity>;

		fn into_iter(self) -> Self::IntoIter {
			self.agents.0.clone().into_iter()
		}
	}

	#[derive(Resource)]
	struct _Agents(Vec<PersistentEntity>);

	fn setup<const N: usize>(agents: [PersistentEntity; N]) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.insert_resource(_Agents(agents.to_vec()));
		app.add_systems(Update, Bar::add_to::<_AgentsParam>);

		app
	}

	#[test]
	fn add_bar() {
		let agents = [PersistentEntity::default(), PersistentEntity::default()];
		let mut app = setup(agents);
		let entities = agents.map(|a| app.world_mut().spawn(a).id());

		app.update();

		assert_eq!(
			[Some(&Bar::default()), Some(&Bar::default())],
			app.world().entity(entities).map(|e| e.get::<Bar>()),
		);
	}

	#[test]
	fn only_insert_new_bars() {
		let agents = [PersistentEntity::default(), PersistentEntity::default()];
		let mut app = setup(agents);
		let entities = [
			app.world_mut().spawn(agents[0]).id(),
			app.world_mut()
				.spawn((
					agents[1],
					Bar {
						scale: 11.,
						..default()
					},
				))
				.id(),
		];

		app.update();

		assert_eq!(
			[
				Some(&Bar::default()),
				Some(&Bar {
					scale: 11.,
					..default()
				}),
			],
			app.world().entity(entities).map(|e| e.get::<Bar>()),
		);
	}
}
