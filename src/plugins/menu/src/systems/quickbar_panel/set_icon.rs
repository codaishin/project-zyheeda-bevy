use crate::components::{icon::Icon, label::UILabel, quickbar_panel::QuickbarPanel};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{
			AssociatedItem,
			AssociatedSystemParam,
			GetFromSystemParam,
			RefInto,
			TryApplyOn,
		},
		handles_loadout::loadout::{NoSkill, SkillIcon, SkillToken},
		handles_localization::Token,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl QuickbarPanel {
	pub(crate) fn set_icon<TAgent, TSlots>(
		mut commands: ZyheedaCommands,
		param: StaticSystemParam<AssociatedSystemParam<TSlots, SlotKey>>,
		panels: Query<PanelComponents>,
		slots: Query<&TSlots, With<TAgent>>,
	) where
		TAgent: Component,
		for<'w, 's> TSlots: Component + GetFromSystemParam<'w, 's, SlotKey>,
		for<'w, 's, 'a> AssociatedItem<'w, 's, TSlots, SlotKey>: RefInto<'a, Result<SkillIcon<'a>, NoSkill>>
			+ RefInto<'a, Result<SkillToken<'a>, NoSkill>>,
	{
		for slots in &slots {
			for (entity, Self { key, .. }, current_icon, current_label) in &panels {
				let Some(item) = slots.get_from_param(&SlotKey::from(*key), &param) else {
					continue;
				};
				let Ok(SkillToken(token)) = item.ref_into() else {
					continue;
				};
				let Ok(SkillIcon(image)) = item.ref_into() else {
					continue;
				};

				commands.try_apply_on(&entity, |mut e| {
					if !loaded(current_icon, image) {
						e.try_insert(Icon::Load(image.clone()));
					}

					if !labeled(current_label, token) {
						e.try_insert(UILabel(token.clone()));
					}
				});
			}
		}
	}
}

type PanelComponents<'a> = (
	Entity,
	&'a QuickbarPanel,
	Option<&'a Icon>,
	Option<&'a UILabel<Token>>,
);

fn loaded(icon: Option<&Icon>, image: &Handle<Image>) -> bool {
	matches!(icon, Some(Icon::Loaded(icon_image)) if icon_image == image)
}

fn labeled(label: Option<&UILabel<Token>>, token: &Token) -> bool {
	matches!(label, Some(UILabel(label_token)) if label_token == token)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{icon::Icon, label::UILabel},
		tools::PanelState,
	};
	use bevy::ecs::system::SystemParam;
	use common::{tools::action_key::slot::PlayerSlot, traits::handles_localization::Token};
	use std::collections::HashMap;
	use testing::{IsChanged, SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, _Item>);

	impl<'w, 's> GetFromSystemParam<'w, 's, SlotKey> for _Slots {
		type TParam = _Param;
		type TItem = _Item;

		fn get_from_param(&self, key: &SlotKey, _: &_Param) -> Option<Self::TItem> {
			self.0.get(key).cloned()
		}
	}

	#[derive(SystemParam)]
	struct _Param;

	#[derive(Clone)]
	struct _Item {
		icon: Option<Handle<Image>>,
		token: Option<Token>,
	}

	impl<'a> From<&'a _Item> for Result<SkillIcon<'a>, NoSkill> {
		fn from(_Item { icon, .. }: &'a _Item) -> Self {
			match icon {
				Some(i) => Ok(SkillIcon(i)),
				None => Err(NoSkill),
			}
		}
	}

	impl<'a> From<&'a _Item> for Result<SkillToken<'a>, NoSkill> {
		fn from(_Item { token, .. }: &'a _Item) -> Self {
			match token {
				Some(t) => Ok(SkillToken(t)),
				None => Err(NoSkill),
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				QuickbarPanel::set_icon::<_Agent, _Slots>,
				IsChanged::<UILabel<Token>>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn insert_icon() {
		let image = new_handle();
		let item = _Item {
			token: Some(Token::from("my item")),
			icon: Some(image.clone()),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		assert_eq!(
			Some(&Icon::Load(image)),
			app.world().entity(panel).get::<Icon>(),
		);
	}

	#[test]
	fn do_not_insert_icon_when_already_loaded() {
		let image = new_handle();
		let item = _Item {
			token: Some(Token::from("my item")),
			icon: Some(image.clone()),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn((
				QuickbarPanel {
					key: PlayerSlot::LOWER_R,
					state: PanelState::Empty,
				},
				Icon::Loaded(image.clone()),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Icon::Loaded(image)),
			app.world().entity(panel).get::<Icon>(),
		);
	}

	#[test]
	fn insert_label() {
		let image = new_handle();
		let item = _Item {
			token: Some(Token::from("my item")),
			icon: Some(image.clone()),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		assert_eq!(
			Some(&UILabel(Token::from("my item"))),
			app.world().entity(panel).get::<UILabel<Token>>(),
		);
	}

	#[test]
	fn do_not_insert_label_when_already_present() {
		let image = new_handle();
		let item = _Item {
			token: Some(Token::from("my item")),
			icon: Some(image.clone()),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(panel).get::<IsChanged<UILabel<Token>>>(),
		);
	}

	#[test]
	fn insert_icon_when_not_already_loaded_but_label_is_already_present() {
		let image = new_handle();
		let item = _Item {
			token: Some(Token::from("my item")),
			icon: Some(image.clone()),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();
		app.world_mut().entity_mut(panel).remove::<Icon>();
		app.update();

		assert_eq!(
			Some(&Icon::Load(image)),
			app.world().entity(panel).get::<Icon>(),
		);
	}

	#[test]
	fn do_nothing_when_slots_lack_agent() {
		let image = new_handle();
		let item = _Item {
			token: Some(Token::from("my item")),
			icon: Some(image.clone()),
		};
		let mut app = setup();
		app.world_mut().spawn(_Slots(HashMap::from([(
			SlotKey::from(PlayerSlot::LOWER_R),
			item.clone(),
		)])));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let panel = app.world().entity(panel);
		assert_eq!(
			(None, None),
			(panel.get::<Icon>(), panel.get::<UILabel::<Token>>()),
		);
	}
}
