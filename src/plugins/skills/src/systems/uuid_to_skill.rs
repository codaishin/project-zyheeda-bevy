use crate::{skills::Skill, states::SkillAssets, traits::TryMap};
use bevy::{
	asset::Assets,
	prelude::{Commands, Component, Entity, Query, Res, State},
};
use uuid::Uuid;

pub(crate) fn uuid_to_skill<TSource, TResult>(
	mut commands: Commands,
	skills: Res<Assets<Skill>>,
	sources: Query<(Entity, &TSource)>,
	skill_assets: Res<State<SkillAssets>>,
) where
	TSource: Component + TryMap<Uuid, Skill, TResult>,
	TResult: Component,
{
	if skill_assets.get() == &SkillAssets::Loading {
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
	TSource: TryMap<Uuid, Skill, TResult>,
{
	source.try_map(|uuid| {
		skills
			.iter()
			.map(|(_, skill)| skill)
			.find(|skill| &skill.id == uuid)
			.cloned()
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::states::SkillAssets;
	use bevy::{
		app::{App, Update},
		asset::Assets,
		prelude::AppExtStates,
		state::app::StatesPlugin,
		utils::default,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Container<T: Sync + Send + 'static>(Option<T>);

	impl TryMap<Uuid, Skill, _Container<Skill>> for _Container<Uuid> {
		fn try_map(&self, mut map_fn: impl FnMut(&Uuid) -> Option<Skill>) -> _Container<Skill> {
			_Container(self.0.and_then(|id| map_fn(&id)))
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_plugins(StatesPlugin);
		app.insert_state(SkillAssets::Loading);
		app.init_resource::<Assets<Skill>>();

		app.add_systems(Update, uuid_to_skill::<_Container<Uuid>, _Container<Skill>>);

		app
	}

	#[test]
	fn map_uuid_to_skill_when_loaded() {
		let id = Uuid::new_v4();
		let mut app = setup();
		app.insert_state(SkillAssets::Loaded);
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
		let id = Uuid::new_v4();
		let mut app = setup();
		app.insert_state(SkillAssets::Loaded);
		app.world_mut().resource_mut::<Assets<Skill>>().add(Skill {
			id,
			name: "my skill".to_owned(),
			..default()
		});
		let agent = app.world_mut().spawn(_Container(Some(id))).id();

		app.update();

		assert_eq!(None, app.world().entity(agent).get::<_Container<Uuid>>())
	}

	#[test]
	fn do_nothing_when_skill_assets_loading() {
		let id = Uuid::new_v4();
		let mut app = setup();
		app.insert_state(SkillAssets::Loading);
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
				agent.get::<_Container<Uuid>>(),
				agent.get::<_Container<Skill>>()
			)
		)
	}
}
