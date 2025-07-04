use crate::traits::insert_ui_content::InsertUiContent;
use bevy::prelude::*;
use common::traits::handles_localization::LocalizeToken;

impl<T> RenderUi for T where T: InsertUiContent + Component {}

pub(crate) trait RenderUi: InsertUiContent + Component + Sized {
	fn render_ui<TLocalization>(
		mut commands: Commands,
		mut localize: ResMut<TLocalization>,
		components: Query<(Entity, &Self), Added<Self>>,
	) where
		TLocalization: LocalizeToken + Resource,
	{
		for (entity, component) in &components {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.with_children(|parent| {
				component.insert_ui_content(localize.as_mut(), parent);
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::relationship::RelatedSpawnerCommands;
	use common::traits::{
		handles_localization::{LocalizationResult, Token, localized::Localized},
		thread_safe::ThreadSafe,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, assert_count, get_children};

	#[derive(Component)]
	struct _Component;

	impl InsertUiContent for _Component {
		fn insert_ui_content<TLocalization>(
			&self,
			localization: &mut TLocalization,
			parent: &mut RelatedSpawnerCommands<ChildOf>,
		) where
			TLocalization: LocalizeToken + ThreadSafe,
		{
			parent.spawn(Text::from(localization.localize_token("a").or_token()));
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Localize {
		mock: Mock_Localize,
	}

	#[automock]
	impl LocalizeToken for _Localize {
		fn localize_token<TToken>(&mut self, token: TToken) -> LocalizationResult
		where
			TToken: Into<Token> + 'static,
		{
			self.mock.localize_token(token)
		}
	}

	fn setup(localize: _Localize) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(localize);
		app.add_systems(Update, _Component::render_ui::<_Localize>);

		app
	}

	#[test]
	fn spawn_content() {
		let localize = _Localize::new().with_mock(|mock| {
			mock.expect_localize_token::<&str>()
				.times(1)
				.with(eq("a"))
				.return_const(LocalizationResult::Ok(Localized::from_string(
					"a localized",
				)));
		});
		let mut app = setup(localize);
		let entity = app.world_mut().spawn(_Component).id();

		app.update();

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(
			Some(&String::from("a localized")),
			child.get::<Text>().map(|Text(s)| s)
		);
	}

	#[test]
	fn spawn_content_only_once() {
		let localize = _Localize::new().with_mock(|mock| {
			mock.expect_localize_token::<&str>()
				.times(1)
				.return_const(LocalizationResult::Ok(Localized::from_string("")));
		});
		let mut app = setup(localize);
		app.world_mut().spawn(_Component);

		app.update();
		app.update();
	}
}
