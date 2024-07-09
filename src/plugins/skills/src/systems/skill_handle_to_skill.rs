use crate::{skills::Skill, traits::TryMap};
use bevy::{
	asset::{AssetEvent, Assets, Handle},
	ecs::system::Res,
	prelude::{Commands, Component, DetectChanges, Entity, EventReader, Query, Ref},
};
use common::{
	errors::{Error, Level},
	traits::try_insert_on::TryInsertOn,
};

pub(crate) fn skill_handle_to_skill<
	TSource: Component + TryMap<Handle<Skill>, Skill, TResult>,
	TResult: Component,
>(
	mut commands: Commands,
	mut events: EventReader<AssetEvent<Skill>>,
	skills: Res<Assets<Skill>>,
	sources: Query<(Entity, Ref<TSource>)>,
) -> Vec<Result<(), Error>> {
	let force_update = events.read().any(added_or_modified);
	let mut errors = vec![];

	for (entity, source) in sources.iter().filter(is_changed_or(force_update)) {
		commands.try_insert_on(entity, source.try_map(get_skill(&skills, &mut errors)));
	}

	errors
}

fn added_or_modified(event: &AssetEvent<Skill>) -> bool {
	matches!(
		event,
		AssetEvent::Added { .. } | AssetEvent::Modified { .. }
	)
}

fn get_skill<'a>(
	skills: &'a Res<Assets<Skill>>,
	errors: &'a mut Vec<Result<(), Error>>,
) -> impl FnMut(&Handle<Skill>) -> Option<Skill> + 'a {
	move |handle| {
		let skill = skills.get(handle).cloned();
		if skill.is_none() {
			errors.push(Err(no_skill_for(handle)));
		}
		skill
	}
}

fn is_changed_or<TSource>(force_update: bool) -> impl FnMut(&(Entity, Ref<TSource>)) -> bool {
	move |(_, source)| source.is_changed() || force_update
}

fn no_skill_for(handle: &Handle<Skill>) -> Error {
	Error {
		msg: format!("No skill found for {handle:?}"),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{AssetEvent, AssetId},
		prelude::IntoSystem,
		utils::{default, Uuid},
	};
	use common::{
		systems::log::test_tools::{fake_log_error_many_recourse, FakeErrorLogManyResource},
		test_tools::utils::SingleThreadedApp,
	};
	use mockall::automock;

	#[automock]
	trait _Counter {
		fn count(&self);
	}

	#[derive(Component)]
	struct _Source {
		handles: Vec<Handle<Skill>>,
		try_map_call_counter: Option<Mock_Counter>,
	}

	impl _Source {
		fn new(handles: Vec<Handle<Skill>>) -> Self {
			Self {
				handles,
				try_map_call_counter: None,
			}
		}
	}

	impl TryMap<Handle<Skill>, Skill, _Result> for _Source {
		fn try_map(&self, map_fn: impl FnMut(&Handle<Skill>) -> Option<Skill>) -> _Result {
			if let Some(counter) = &self.try_map_call_counter {
				counter.count();
			}
			_Result(self.handles.iter().filter_map(map_fn).collect())
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Result(Vec<Skill>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<Skill>>();
		app.add_event::<AssetEvent<Skill>>();
		app.add_systems(
			Update,
			skill_handle_to_skill::<_Source, _Result>.pipe(fake_log_error_many_recourse),
		);

		app
	}

	fn expect_try_map_called_times(times: usize) -> Mock_Counter {
		let mut counter = Mock_Counter::default();
		counter.expect_count().times(times).return_const(());
		counter
	}

	#[test]
	fn map_handles_to_skills() {
		let mut app = setup();

		let skills = vec![
			app.world.resource_mut::<Assets<Skill>>().add(Skill {
				name: "my skill a".to_owned(),
				..default()
			}),
			app.world.resource_mut::<Assets<Skill>>().add(Skill {
				name: "my skill b".to_owned(),
				..default()
			}),
			app.world.resource_mut::<Assets<Skill>>().add(Skill {
				name: "my skill c".to_owned(),
				..default()
			}),
		];
		let source = app.world.spawn(_Source::new(skills.clone())).id();

		app.update();

		let source = app.world.entity(source);

		assert_eq!(
			Some(&_Result(vec![
				Skill {
					name: "my skill a".to_owned(),
					..default()
				},
				Skill {
					name: "my skill b".to_owned(),
					..default()
				},
				Skill {
					name: "my skill c".to_owned(),
					..default()
				}
			])),
			source.get::<_Result>()
		);
	}

	#[test]
	fn map_only_when_source_changed() {
		let mut app = setup();

		let skills = vec![app.world.resource_mut::<Assets<Skill>>().add(Skill {
			name: "my skill a".to_owned(),
			..default()
		})];
		let other_skills = vec![app.world.resource_mut::<Assets<Skill>>().add(Skill {
			name: "my other skill a".to_owned(),
			..default()
		})];

		let source = app
			.world
			.spawn(_Source {
				handles: skills,
				try_map_call_counter: Some(expect_try_map_called_times(2)),
			})
			.id();

		app.update();
		app.update();

		app.world
			.entity_mut(source)
			.get_mut::<_Source>()
			.unwrap()
			.handles = other_skills;

		app.update();
	}

	#[test]
	fn map_also_when_skills_added() {
		let mut app = setup();

		let skill = app.world.resource_mut::<Assets<Skill>>().add(Skill {
			name: "my skill a".to_owned(),
			..default()
		});

		app.world.spawn(_Source {
			handles: vec![skill.clone()],
			try_map_call_counter: Some(expect_try_map_called_times(2)),
		});

		app.update();
		app.update();

		app.world
			.send_event(AssetEvent::<Skill>::Added { id: skill.into() });

		app.update();
	}

	#[test]
	fn map_also_when_skills_modified() {
		let mut app = setup();

		let skill = app.world.resource_mut::<Assets<Skill>>().add(Skill {
			name: "my skill a".to_owned(),
			..default()
		});

		app.world.spawn(_Source {
			handles: vec![skill.clone()],
			try_map_call_counter: Some(expect_try_map_called_times(2)),
		});

		app.update();
		app.update();

		app.world
			.send_event(AssetEvent::<Skill>::Modified { id: skill.into() });

		app.update();
	}

	#[test]
	fn log_error() {
		let mut app = setup();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});

		app.world.spawn(_Source::new(vec![handle.clone()]));

		app.update();

		let errors = app.world.get_resource::<FakeErrorLogManyResource>();

		assert_eq!(
			Some(&FakeErrorLogManyResource(vec![no_skill_for(&handle)])),
			errors
		);
	}
}
