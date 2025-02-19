use crate::{
	item::Item,
	skills::Skill,
	tools::{cache::Cache, loadout_item::LoadoutItem},
	traits::loadout_key::LoadoutKey,
};
use bevy::prelude::*;
use common::{tools::change::Change, traits::iterate::Iterate};
use std::hash::Hash;

impl<T> LoadoutDescriptor for T {}

pub(crate) trait LoadoutDescriptor {
	fn describe_loadout_for<TAgent>(
		containers: Query<&Self, (With<TAgent>, Changed<Self>)>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) -> Change<Cache<Self::TKey, LoadoutItem>>
	where
		for<'a> Self: LoadoutKey
			+ Iterate<TItem<'a> = (Self::TKey, &'a Option<Handle<Item>>)>
			+ Component
			+ Sized,
		TAgent: Component,
		Self::TKey: Eq + Hash + Copy,
	{
		let Ok(container) = containers.get_single() else {
			return Change::None;
		};

		let map = container
			.iterate()
			.filter_map(|(key, handle)| {
				let handle = handle.as_ref()?;
				let item = items.get(handle)?;
				let handle = item.skill.as_ref()?;
				let skill = skills.get(handle)?;
				let skill_icon = skill.icon.clone();

				Some((
					key,
					LoadoutItem {
						name: item.name.clone(),
						skill_icon,
					},
				))
			})
			.collect();

		Change::Some(Cache(map))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::{new_handle, SingleThreadedApp};
	use std::collections::HashMap;

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key;

	#[derive(Component)]
	struct _Loadout(Option<Handle<Item>>);

	impl LoadoutKey for _Loadout {
		type TKey = _Key;
	}

	impl Iterate for _Loadout {
		type TItem<'a>
			= (_Key, &'a Option<Handle<Item>>)
		where
			Self: 'a;

		fn iterate(&self) -> impl Iterator<Item = Self::TItem<'_>> {
			let _Loadout(item) = self;
			[(_Key, item)].into_iter()
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Change<Cache<_Key, LoadoutItem>>);

	fn setup(item_name: &'static str, skill_icon: Option<Handle<Image>>) -> (App, Handle<Item>) {
		let mut app = App::new().single_threaded(Update);
		let mut skills = Assets::default();
		let mut items = Assets::default();

		let skill = skills.add(Skill {
			icon: skill_icon,
			..default()
		});
		let item = items.add(Item {
			name: item_name.to_owned(),
			skill: Some(skill),
			..default()
		});

		app.insert_resource(skills);
		app.insert_resource(items);
		app.add_systems(
			Update,
			_Loadout::describe_loadout_for::<_Agent>
				.pipe(|In(c), mut commands: Commands| commands.insert_resource(_Result(c))),
		);

		(app, item)
	}

	#[test]
	fn return_description() {
		let icon = Some(new_handle());
		let (mut app, item) = setup("my item", icon.clone());
		app.world_mut().spawn((_Agent, _Loadout(Some(item))));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				_Key,
				LoadoutItem {
					name: "my item".to_owned(),
					skill_icon: icon,
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_none_when_not_changed() {
		let (mut app, item) = setup("my item", Some(new_handle()));
		app.world_mut().spawn((_Agent, _Loadout(Some(item))));

		app.update();
		app.update();

		assert_eq!(
			Some(&_Result(Change::None)),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_some_when_changed() {
		let (mut app, item) = setup("my item", Some(new_handle()));
		let agent = app.world_mut().spawn((_Agent, _Loadout(Some(item)))).id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Loadout>()
			.as_deref_mut();
		app.update();

		assert!(matches!(
			app.world().get_resource::<_Result>(),
			Some(&_Result(Change::Some(_)))
		));
	}

	#[test]
	fn return_none_when_no_agent() {
		let (mut app, item) = setup("my item", Some(new_handle()));
		app.world_mut().spawn(_Loadout(Some(item)));

		app.update();

		assert_eq!(
			Some(&_Result(Change::None)),
			app.world().get_resource::<_Result>()
		);
	}
}
