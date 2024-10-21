use crate::{
	skills::{Skill, SkillId},
	traits::TryMap,
};
use bevy::{
	asset::Assets,
	prelude::{Commands, Component, Entity, Query, Res, State},
};
use common::states::{AssetLoadState, LoadState};

pub(crate) fn uuid_to_skill<TSource, TResult>(
	mut commands: Commands,
	skills: Res<Assets<Skill>>,
	sources: Query<(Entity, &TSource)>,
	state: Res<State<AssetLoadState<Skill>>>,
) where
	TSource: Component + TryMap<SkillId, Skill, TResult>,
	TResult: Component,
{
	if **state.get() == LoadState::Loading {
		return;
	}

	for (entity, source) in &sources {
		let result = apply_map(source, &skills);

		let Some(mut entity) = commands.get_entity(entity) else {
			continue;
		};

		entity.try_insert(result);
		entity.remove::<TSource>();
	}
}

fn apply_map<TSource, TResult>(source: &TSource, skills: &Res<Assets<Skill>>) -> TResult
where
	TSource: TryMap<SkillId, Skill, TResult>,
{
	source.try_map(|skill_id| {
		skills
			.iter()
			.map(|(_, skill)| skill)
			.find(|skill| &skill.id == skill_id)
			.cloned()
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::Assets,
		prelude::AppExtStates,
		state::app::StatesPlugin,
		utils::default,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use uuid::Uuid;

	#[derive(Component, Debug, PartialEq)]
	struct _Container<T: Sync + Send + 'static>(Option<T>);

	impl TryMap<SkillId, Skill, _Container<Skill>> for _Container<SkillId> {
		fn try_map(&self, mut map_fn: impl FnMut(&SkillId) -> Option<Skill>) -> _Container<Skill> {
			_Container(self.0.and_then(|id| map_fn(&id)))
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_plugins(StatesPlugin);
		app.insert_state(AssetLoadState::<Skill>::new(LoadState::Loading));
		app.init_resource::<Assets<Skill>>();

		app.add_systems(
			Update,
			uuid_to_skill::<_Container<SkillId>, _Container<Skill>>,
		);

		app
	}

	#[test]
	fn map_uuid_to_skill_when_loaded() {
		let id = SkillId(Uuid::new_v4());
		let mut app = setup();
		app.insert_state(AssetLoadState::<Skill>::new(LoadState::Loaded));
		app.world_mut().resource_mut::<Assets<Skill>>().add(Skill {
			id,
			name: "my skill".to_owned(),
			..default()
		});
		let agent = app.world_mut().spawn(_Container(Some(id))).id();

		app.update();

		assert_eq!(
			Some(&_Container(Some(Skill {
				id,
				name: "my skill".to_owned(),
				..default()
			}))),
			app.world().entity(agent).get::<_Container<Skill>>()
		)
	}

	#[test]
	fn remove_source() {
		let id = SkillId(Uuid::new_v4());
		let mut app = setup();
		app.insert_state(AssetLoadState::<Skill>::new(LoadState::Loaded));
		app.world_mut().resource_mut::<Assets<Skill>>().add(Skill {
			id,
			name: "my skill".to_owned(),
			..default()
		});
		let agent = app.world_mut().spawn(_Container(Some(id))).id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Container<SkillId>>())
	}

	#[test]
	fn do_nothing_when_skill_assets_loading() {
		let id = SkillId(Uuid::new_v4());
		let mut app = setup();
		app.insert_state(AssetLoadState::<Skill>::new(LoadState::Loading));
		app.world_mut().resource_mut::<Assets<Skill>>().add(Skill {
			id,
			name: "my skill".to_owned(),
			..default()
		});
		let agent = app.world_mut().spawn(_Container(Some(id))).id();

		app.update();

		let agent = app.world().entity(agent);
		assert_eq!(
			(Some(&_Container(Some(id))), None),
			(
				agent.get::<_Container<SkillId>>(),
				agent.get::<_Container<Skill>>()
			)
		)
	}
}
