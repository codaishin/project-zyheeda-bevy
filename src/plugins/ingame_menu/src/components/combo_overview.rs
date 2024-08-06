use super::{
	key_code_text_insert_command::KeyCodeTextInsertCommandBundle,
	skill_descriptor::{DropdownTrigger, SkillDescriptor, Vertical},
	tooltip::Tooltip,
	AppendSkillCommand,
	DeleteSkill,
	KeySelectDropdownInsertCommand,
	ReKeyCommand,
	SkillSelectDropdownInsertCommand,
};
use crate::{
	tools::{Dimensions, Pixel},
	traits::{
		colors::DEFAULT_PANEL_COLORS,
		get_node::GetNode,
		instantiate_content_on::InstantiateContentOn,
		CombosDescriptor,
		UpdateCombos,
	},
};
use bevy::{
	asset::Handle,
	color::Color,
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::{Bundle, Component},
	render::texture::Image,
	text::TextStyle,
	ui::{
		node_bundles::{ButtonBundle, NodeBundle, TextBundle},
		AlignItems,
		FlexDirection,
		JustifyContent,
		PositionType,
		Style,
		UiImage,
		UiRect,
		Val,
	},
	utils::default,
};
use skills::items::slot_key::SlotKey;

#[derive(Component, Default)]
pub(crate) struct ComboOverview(CombosDescriptor);

pub(crate) trait SkillButtonBundle {
	fn with<T: Clone + Sync + Send + 'static>(self, descriptor: SkillDescriptor<T>) -> impl Bundle;
}

impl SkillButtonBundle for ButtonBundle {
	fn with<T: Clone + Sync + Send + 'static>(self, descriptor: SkillDescriptor<T>) -> impl Bundle {
		(self, Tooltip::new(descriptor.skill.clone()), descriptor)
	}
}

impl ComboOverview {
	pub const MODIFY_BUTTON_OFFSET: Pixel = Pixel(-12.0);
	pub const SKILL_BUTTON_DIMENSIONS: Dimensions = Dimensions {
		width: Pixel(65.0),
		height: Pixel(65.0),
		border: Pixel(0.0),
	};
	pub const KEY_BUTTON_DIMENSIONS: Dimensions = Dimensions {
		width: Pixel(50.0),
		height: Pixel(25.0),
		border: Pixel(2.0),
	};
	pub const MODIFY_BUTTON_DIMENSIONS: Dimensions = Dimensions {
		width: Pixel(20.0),
		height: Pixel(25.0),
		border: Pixel(2.0),
	};

	pub(crate) fn skill_container_bundle() -> impl Bundle {
		NodeBundle {
			style: Style {
				margin: UiRect::all(Val::Px(10.0)),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn skill_button_bundle(icon: Handle<Image>) -> impl SkillButtonBundle + Bundle {
		ButtonBundle {
			style: Style {
				width: Val::from(Self::SKILL_BUTTON_DIMENSIONS.width),
				height: Val::from(Self::SKILL_BUTTON_DIMENSIONS.height),
				border: UiRect::from(Self::SKILL_BUTTON_DIMENSIONS.border),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.filled.into(),
			image: UiImage::new(icon),
			..default()
		}
	}

	pub(crate) fn skill_key_button_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				top: Val::from(Self::MODIFY_BUTTON_OFFSET),
				right: Val::from(Self::MODIFY_BUTTON_OFFSET),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn delete_button_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				left: Val::from(Self::MODIFY_BUTTON_OFFSET),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn append_button_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				right: Val::from(Self::MODIFY_BUTTON_OFFSET),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn skill_key_button_bundle() -> impl Bundle {
		ButtonBundle {
			style: Style {
				width: Val::from(Self::KEY_BUTTON_DIMENSIONS.width),
				height: Val::from(Self::KEY_BUTTON_DIMENSIONS.height),
				border: UiRect::from(Self::KEY_BUTTON_DIMENSIONS.border),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.filled.into(),
			border_color: DEFAULT_PANEL_COLORS.text.into(),
			..default()
		}
	}

	pub(crate) fn modify_button_bundle() -> impl Bundle {
		ButtonBundle {
			style: Style {
				width: Val::from(Self::MODIFY_BUTTON_DIMENSIONS.width),
				height: Val::from(Self::MODIFY_BUTTON_DIMENSIONS.height),
				border: UiRect::from(Self::MODIFY_BUTTON_DIMENSIONS.border),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.filled.into(),
			border_color: DEFAULT_PANEL_COLORS.text.into(),
			..default()
		}
	}

	pub(crate) fn skill_key_text(key: SlotKey) -> impl Bundle {
		KeyCodeTextInsertCommandBundle::new(
			key,
			TextStyle {
				font_size: 15.,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}

	pub(crate) fn modify_button_text(key: &str) -> impl Bundle {
		TextBundle::from_section(
			key,
			TextStyle {
				font_size: 15.,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}
}

impl UpdateCombos for ComboOverview {
	fn update_combos(&mut self, combos: CombosDescriptor) {
		self.0 = combos
	}
}

impl GetNode for ComboOverview {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				top: Val::Px(30.0),
				left: Val::Px(30.0),
				right: Val::Px(30.0),
				bottom: Val::Px(30.0),
				flex_direction: FlexDirection::Column,
				..default()
			},
			background_color: Color::srgba(0.5, 0.5, 0.5, 0.5).into(),
			..default()
		}
	}
}

impl InstantiateContentOn for ComboOverview {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		add_title(parent, "Combos");
		add_combo_list(parent, self);
	}
}

fn add_title(parent: &mut ChildBuilder, title: &str) {
	parent
		.spawn(NodeBundle {
			style: Style {
				margin: UiRect::all(Val::Px(10.)),
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			parent.spawn(TextBundle::from_section(
				title,
				TextStyle {
					font_size: 40.0,
					color: DEFAULT_PANEL_COLORS.text,
					..default()
				},
			));
		});
}

fn add_combo_list(parent: &mut ChildBuilder, combo_overview: &ComboOverview) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			for combo in &combo_overview.0 {
				add_combo(parent, combo);
			}
		});
}

fn add_combo(parent: &mut ChildBuilder, combo: &[SkillDescriptor<DropdownTrigger>]) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				margin: UiRect::top(Val::Px(10.0)),
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			let last = combo.len() - 1;

			for skill in &combo[..last] {
				add_skill(parent, skill, &[with_key_button]);
			}

			let Some(last_skill) = combo.last() else {
				return;
			};

			add_skill(
				parent,
				last_skill,
				&[with_key_button, with_append_button, with_delete_button],
			);
		});
}

fn add_skill(
	parent: &mut ChildBuilder,
	descriptor: &SkillDescriptor<DropdownTrigger>,
	additional_buttons: &[fn(&SkillDescriptor<DropdownTrigger>, &mut ChildBuilder)],
) {
	let skill_icon = descriptor.skill.icon.clone().unwrap_or_default();

	parent
		.spawn(ComboOverview::skill_container_bundle())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::skill_button_bundle(skill_icon).with(descriptor.clone()),
					SkillSelectDropdownInsertCommand::<SlotKey, Vertical>::new(
						descriptor.key_path.clone(),
					),
				))
				.with_children(|parent| {
					for additional_button in additional_buttons {
						additional_button(descriptor, parent);
					}
				});
		});
}

fn with_key_button(descriptor: &SkillDescriptor<DropdownTrigger>, parent: &mut ChildBuilder) {
	let Some(skill_key) = descriptor.key_path.last() else {
		return;
	};

	parent
		.spawn(ComboOverview::skill_key_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::skill_key_button_bundle(),
					KeySelectDropdownInsertCommand {
						extra: ReKeyCommand { ignore: *skill_key },
						key_path: descriptor.key_path.clone(),
					},
				))
				.with_children(|parent| {
					parent.spawn(ComboOverview::skill_key_text(*skill_key));
				});
		});
}

fn with_append_button(descriptor: &SkillDescriptor<DropdownTrigger>, parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::append_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::modify_button_bundle(),
					KeySelectDropdownInsertCommand {
						extra: AppendSkillCommand,
						key_path: descriptor.key_path.clone(),
					},
				))
				.with_children(|parent| {
					parent.spawn(ComboOverview::modify_button_text(">"));
				});
		});
}

fn with_delete_button(descriptor: &SkillDescriptor<DropdownTrigger>, parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::delete_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::modify_button_bundle(),
					DeleteSkill {
						key_path: descriptor.key_path.clone(),
					},
				))
				.with_children(|parent| {
					parent.spawn(ComboOverview::modify_button_text("<"));
				});
		});
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetId;
	use common::components::Side;
	use skills::skills::Skill;
	use uuid::Uuid;

	#[test]
	fn update_combos() {
		let combos = vec![vec![SkillDescriptor::<DropdownTrigger>::new(
			Skill {
				name: "my skill".to_owned(),
				icon: Some(Handle::Weak(AssetId::Uuid {
					uuid: Uuid::new_v4(),
				})),
				..default()
			},
			vec![SlotKey::Hand(Side::Main)],
		)]];
		let mut combo_overview = ComboOverview::default();
		combo_overview.update_combos(combos.clone());

		assert_eq!(combos, combo_overview.0)
	}
}
