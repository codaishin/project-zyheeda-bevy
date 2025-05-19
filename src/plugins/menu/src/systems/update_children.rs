use crate::traits::insert_ui_content::InsertUiContent;
use bevy::prelude::*;
use common::traits::handles_localization::LocalizeToken;

pub(crate) fn update_children<TComponent, TLocalization>(
	mut commands: Commands,
	components: Query<(Entity, &TComponent), Changed<TComponent>>,
	mut localization_server: ResMut<TLocalization>,
) where
	TComponent: InsertUiContent + Component,
	TLocalization: LocalizeToken + Resource,
{
	for (entity, component) in &components {
		let Ok(mut entity) = commands.get_entity(entity) else {
			continue;
		};
		entity.despawn_related::<Children>();
		entity.with_children(|parent| {
			component.insert_ui_content(localization_server.as_mut(), parent)
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::relationship::RelatedSpawnerCommands;
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{
			handles_localization::{LocalizationResult, Token, localized::Localized},
			thread_safe::ThreadSafe,
		},
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Child(Localized);

	impl From<&str> for _Child {
		fn from(value: &str) -> Self {
			Self(Localized::from(value))
		}
	}

	#[derive(Component)]
	struct _Component(&'static str);

	impl InsertUiContent for _Component {
		fn insert_ui_content<TLocalization>(
			&self,
			localize: &mut TLocalization,
			parent: &mut RelatedSpawnerCommands<ChildOf>,
		) where
			TLocalization: LocalizeToken + ThreadSafe,
		{
			parent.spawn(_Child(localize.localize_token("A").or_string(|| "??")));
			parent.spawn(_Child(localize.localize_token("B").or_string(|| "??")));
			parent.spawn(_Child(localize.localize_token("C").or_string(|| "??")));
		}
	}

	#[derive(Resource, Debug, PartialEq, Default)]
	struct _Localization;

	impl LocalizeToken for _Localization {
		fn localize_token<TToken>(&mut self, token: TToken) -> LocalizationResult
		where
			TToken: Into<Token> + 'static,
		{
			match token.into() {
				t if t == Token::from("A") => LocalizationResult::Ok(Localized::from("Token A")),
				t if t == Token::from("B") => LocalizationResult::Ok(Localized::from("Token B")),
				t if t == Token::from("C") => LocalizationResult::Ok(Localized::from("Token C")),
				t => LocalizationResult::Error(t.failed()),
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_Localization>();
		app.add_systems(Update, update_children::<_Component, _Localization>);

		app
	}

	#[test]
	fn render_children() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Component("My Component")).id();

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| Some((e.get::<ChildOf>()?.parent(), e.get::<_Child>()?)));

		assert_eq!(
			vec![
				(parent, &_Child::from("Token A")),
				(parent, &_Child::from("Token B")),
				(parent, &_Child::from("Token C")),
			],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn remove_previous_children() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Component("My Component"))
			.with_children(|parent| {
				parent.spawn(_Child::from("Previous A"));
				parent.spawn(_Child::from("Previous B"));
			});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![
				&_Child::from("Token A"),
				&_Child::from("Token B"),
				&_Child::from("Token C"),
			],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn remove_previous_children_recursively() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Component("My Component"))
			.with_children(|parent| {
				parent
					.spawn(_Child::from("Previous A"))
					.with_children(|parent| {
						parent.spawn(_Child::from("Previous A Child"));
					});
			});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![
				&_Child::from("Token A"),
				&_Child::from("Token B"),
				&_Child::from("Token C"),
			],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn only_work_when_added() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Component("My Component")).id();

		app.update();

		app.world_mut().entity_mut(parent).with_children(|parent| {
			parent.spawn(_Child::from("Do not remove"));
		});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![
				&_Child::from("Token A"),
				&_Child::from("Token B"),
				&_Child::from("Token C"),
				&_Child::from("Do not remove"),
			],
			children.collect::<Vec<_>>()
		)
	}

	#[test]
	fn work_when_changed() {
		let mut app = setup();
		let parent = app.world_mut().spawn(_Component("My Component")).id();

		app.update();

		let mut parent = app.world_mut().entity_mut(parent);
		parent.get_mut::<_Component>().unwrap().0 = "My changed Component";
		parent.with_children(|parent| {
			parent.spawn(_Child::from("Do remove"));
		});

		app.update();

		let children = app
			.world()
			.iter_entities()
			.filter_map(|e| e.get::<_Child>());

		assert_eq!(
			vec![
				&_Child::from("Token A"),
				&_Child::from("Token B"),
				&_Child::from("Token C"),
			],
			children.collect::<Vec<_>>()
		)
	}
}
