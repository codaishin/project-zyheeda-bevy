use crate::components::quickbar_panel::QuickbarPanel;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::{GetFromParam, RefAs, RefInto},
		handles_loadout::{SkillIcon, SkillToken},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl QuickbarPanel {
	pub(crate) fn set_image<TAgent, TSlots>(
		mut commands: ZyheedaCommands,
		param: StaticSystemParam<<TSlots as GetFromParam<'_, '_, PlayerSlot>>::TParam>,
		panels: Query<(Entity, &Self)>,
		slots: Query<&TSlots>,
	) where
		TAgent: Component,
		for<'w, 's> TSlots: Component + GetFromParam<'w, 's, PlayerSlot>,
		for<'w, 's, 'a> Value<'w, 's, TSlots>:
			RefInto<'a, SkillIcon<'a>> + RefInto<'a, SkillToken<'a>>,
	{
		for slots in &slots {
			for (entity, Self { key, .. }) in &panels {
				let item = slots.get_from_param(key, &param);
				let SkillIcon(Some(image)) = item.ref_as::<SkillIcon>() else {
					continue;
				};
			}
		}
	}
}

type Value<'w, 's, TSlots> = <TSlots as GetFromParam<'w, 's, PlayerSlot>>::TValue;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tools::PanelState;
	use bevy::ecs::system::SystemParam;
	use common::traits::handles_localization::Token;
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, assert_count, get_children, new_handle};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Slots(HashMap<PlayerSlot, _Item>);

	impl<'w, 's> GetFromParam<'w, 's, PlayerSlot> for _Slots {
		type TParam = _Param;
		type TValue = _Item;

		fn get_from_param(&self, key: &PlayerSlot, _: &_Param) -> Self::TValue {
			match self.0.get(key) {
				Some(item) => item.clone(),
				None => _Item {
					token: None,
					icon: None,
				},
			}
		}
	}

	#[derive(SystemParam)]
	struct _Param;

	#[derive(Clone)]
	struct _Item {
		icon: Option<Handle<Image>>,
		token: Option<Token>,
	}

	impl<'a> From<&'a _Item> for SkillIcon<'a> {
		fn from(_Item { icon, .. }: &'a _Item) -> Self {
			Self(icon.as_ref())
		}
	}

	impl<'a> From<&'a _Item> for SkillToken<'a> {
		fn from(_Item { token, .. }: &'a _Item) -> Self {
			Self(token.as_ref())
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, QuickbarPanel::set_image::<_Agent, _Slots>);

		app
	}

	#[test]
	fn insert_icon() {
		let image = new_handle();
		let item = _Item {
			token: Some(Token::from("my item")),
			icon: Some(image),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(PlayerSlot::LOWER_R, item.clone())])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		// let [child] = assert_count!(1, get_children!(app, panel));
		// assert_eq!(Some(&image), child.get::<ImageNode>().map(|i| &i.image));
	}
}
