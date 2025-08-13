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
	fn describe_loadout_for<TAgent, TTargetKey>(
		containers: Query<&Self, (With<TAgent>, Changed<Self>)>,
		items: Res<Assets<Item>>,
		skills: Res<Assets<Skill>>,
	) -> Change<Cache<TTargetKey, LoadoutItem>>
	where
		for<'a> Self: LoadoutKey
			+ Iterate<'a, TItem = (Self::TKey, &'a Option<Handle<Item>>)>
			+ Component
			+ Sized,
		TAgent: Component,
		TTargetKey: TryFrom<Self::TKey> + Eq + Hash + Copy,
	{
		let Ok(container) = containers.single() else {
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
					TTargetKey::try_from(key).ok()?,
					LoadoutItem {
						token: item.token.clone(),
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
	use common::{tools::action_key::IsNot, traits::handles_localization::Token};
	use std::{collections::HashMap, slice::Iter};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _Key {
		TargetKey,
		NoTargetKey,
	}

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _TargetKey;

	impl TryFrom<_Key> for _TargetKey {
		type Error = IsNot<_TargetKey>;

		fn try_from(key: _Key) -> Result<Self, Self::Error> {
			match key {
				_Key::TargetKey => Ok(_TargetKey),
				_Key::NoTargetKey => Err(IsNot::key()),
			}
		}
	}

	#[derive(Component)]
	struct _Loadout(Vec<(_Key, Option<Handle<Item>>)>);

	impl LoadoutKey for _Loadout {
		type TKey = _Key;
	}

	impl<'a> Iterate<'a> for _Loadout {
		type TItem = (_Key, &'a Option<Handle<Item>>);
		type TIter = _Iter<'a>;

		fn iterate(&'a self) -> Self::TIter {
			let _Loadout(items) = self;
			_Iter { it: items.iter() }
		}
	}

	struct _Iter<'a> {
		it: Iter<'a, (_Key, Option<Handle<Item>>)>,
	}

	impl<'a> Iterator for _Iter<'a> {
		type Item = (_Key, &'a Option<Handle<Item>>);

		fn next(&mut self) -> Option<Self::Item> {
			let (key, item) = self.it.next()?;
			Some((*key, item))
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Change<Cache<_TargetKey, LoadoutItem>>);

	fn setup(item_token: &'static str, skill_icon: Option<Handle<Image>>) -> (App, Handle<Item>) {
		let mut app = App::new().single_threaded(Update);
		let mut skills = Assets::default();
		let mut items = Assets::default();

		let skill = skills.add(Skill {
			icon: skill_icon,
			..default()
		});
		let item = items.add(Item {
			token: Token::from(item_token),
			skill: Some(skill),
			..default()
		});

		app.insert_resource(skills);
		app.insert_resource(items);
		app.add_systems(
			Update,
			_Loadout::describe_loadout_for::<_Agent, _TargetKey>
				.pipe(|In(c), mut commands: Commands| commands.insert_resource(_Result(c))),
		);

		(app, item)
	}

	#[test]
	fn return_description() {
		let icon = Some(new_handle());
		let (mut app, item) = setup("my item", icon.clone());
		app.world_mut()
			.spawn((_Agent, _Loadout(vec![(_Key::TargetKey, Some(item))])));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				_TargetKey,
				LoadoutItem {
					token: Token::from("my item"),
					skill_icon: icon,
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn return_none_when_not_changed() {
		let (mut app, item) = setup("my item", Some(new_handle()));
		app.world_mut()
			.spawn((_Agent, _Loadout(vec![(_Key::TargetKey, Some(item))])));

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
		let agent = app
			.world_mut()
			.spawn((_Agent, _Loadout(vec![(_Key::TargetKey, Some(item))])))
			.id();

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
		app.world_mut()
			.spawn(_Loadout(vec![(_Key::TargetKey, Some(item))]));

		app.update();

		assert_eq!(
			Some(&_Result(Change::None)),
			app.world().get_resource::<_Result>()
		);
	}

	#[test]
	fn ignore_key_that_cannot_be_mapped() {
		let icon = Some(new_handle());
		let (mut app, item) = setup("my item", icon.clone());
		app.world_mut().spawn((
			_Agent,
			_Loadout(vec![
				(_Key::TargetKey, Some(item)),
				(_Key::NoTargetKey, Some(new_handle())),
			]),
		));

		app.update();

		assert_eq!(
			Some(&_Result(Change::Some(Cache(HashMap::from([(
				_TargetKey,
				LoadoutItem {
					token: Token::from("my item"),
					skill_icon: icon,
				}
			)]))))),
			app.world().get_resource::<_Result>()
		);
	}
}
