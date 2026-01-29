use crate::components::label::UILabel;
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_localization::{Localize, Token},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl UILabel {
	pub(crate) fn localize<TLocalizer>(
		on_insert: On<Insert, UILabel<Token>>,
		mut commands: ZyheedaCommands,
		labels: Query<&UILabel<Token>>,
		localizer: Res<TLocalizer>,
	) where
		TLocalizer: Resource + Localize,
	{
		let entity = on_insert.entity;
		let Ok(UILabel(token)) = labels.get(entity) else {
			return;
		};

		commands.try_apply_on(&entity, |mut e| {
			let localized = localizer.localize(token).or_token();
			e.try_insert(UILabel(localized));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_localization::{
		LocalizationResult,
		Localize,
		localized::Localized,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Localizer {
		mock: Mock_Localizer,
	}

	#[automock]
	impl Localize for _Localizer {
		fn localize(&self, token: &Token) -> LocalizationResult {
			self.mock.localize(token)
		}
	}

	fn setup(localize: _Localizer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(localize);
		app.add_observer(UILabel::localize::<_Localizer>);

		app
	}

	#[test]
	fn insert_label() {
		let mut app = setup(_Localizer::new().with_mock(|mock| {
			mock.expect_localize()
				.return_const(Localized::from("my label"));
		}));

		let label = app.world_mut().spawn(UILabel(Token::from("my token")));

		assert_eq!(
			Some(&UILabel(Localized::from("my label"))),
			label.get::<UILabel>(),
		);
	}

	#[test]
	fn fallback_to_token() {
		let mut app = setup(_Localizer::new().with_mock(|mock| {
			mock.expect_localize()
				.return_const(Token::from("my token").failed());
		}));

		let label = app.world_mut().spawn(UILabel(Token::from("my token")));

		assert_eq!(
			Some(&UILabel(Localized::from("my token"))),
			label.get::<UILabel>(),
		);
	}

	#[test]
	fn use_label_token() {
		let mut app = setup(_Localizer::new().with_mock(|mock| {
			mock.expect_localize()
				.times(1)
				.with(eq(Token::from("my token")))
				.return_const(Localized::from("my label"));
		}));

		app.world_mut().spawn(UILabel(Token::from("my token")));
	}

	#[test]
	fn re_insert_label() {
		let mut app = setup(_Localizer::new().with_mock(|mock| {
			mock.expect_localize()
				.with(eq(Token::from("my token")))
				.return_const(Localized::from("my label"));
			mock.expect_localize()
				.with(eq(Token::from("my token 2")))
				.return_const(Localized::from("my label 2"));
		}));

		let mut label = app.world_mut().spawn(UILabel(Token::from("my token")));
		label.insert(UILabel(Token::from("my token 2")));

		assert_eq!(
			Some(&UILabel(Localized::from("my label 2"))),
			label.get::<UILabel>(),
		);
	}
}
