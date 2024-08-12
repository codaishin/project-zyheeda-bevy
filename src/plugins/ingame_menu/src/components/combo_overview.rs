use super::{
	key_code_text_insert_command::KeyCodeTextInsertCommandBundle,
	skill_button::{DropdownTrigger, SkillButton, Vertical},
	tooltip::Tooltip,
	AppendSkillCommand,
	DeleteSkill,
	KeySelectDropdownInsertCommand,
	SkillSelectDropdownInsertCommand,
};
use crate::{
	tools::{Dimensions, Pixel},
	traits::{
		colors::DEFAULT_PANEL_COLORS,
		combo_tree_layout::{ComboTreeElement, ComboTreeLayout, Symbol},
		get_node::GetNode,
		instantiate_content_on::InstantiateContentOn,
		LoadUi,
		UpdateCombosView,
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
		ZIndex,
	},
	utils::default,
};
use common::traits::load_asset::{LoadAsset, Path};
use skills::{items::slot_key::SlotKey, skills::Skill};

#[derive(Component, Default, Debug, PartialEq)]
pub(crate) struct ComboOverview {
	new_skill_icon: Handle<Image>,
	layout: ComboTreeLayout,
}

impl<TAssetServer> LoadUi<TAssetServer> for ComboOverview
where
	TAssetServer: LoadAsset,
{
	fn load_ui(images: &mut TAssetServer) -> Self {
		ComboOverview {
			new_skill_icon: images.load_asset(Path::from("icons/empty.png")),
			..default()
		}
	}
}

pub(crate) trait SkillButtonBundle {
	fn with<T: Clone + Sync + Send + 'static>(self, descriptor: SkillButton<T>) -> impl Bundle;
}

impl SkillButtonBundle for ButtonBundle {
	fn with<T: Clone + Sync + Send + 'static>(self, descriptor: SkillButton<T>) -> impl Bundle {
		(self, Tooltip::new(descriptor.skill.clone()), descriptor)
	}
}

impl ComboOverview {
	pub const BUTTON_FONT_SIZE: f32 = 15.;
	pub const SKILL_ICON_MARGIN: Pixel = Pixel(15.);
	pub const MODIFY_BUTTON_OFFSET: Pixel = Pixel(-12.0);
	pub const SYMBOL_WIDTH: Pixel = Pixel(5.);
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
		width: Pixel(25.0),
		height: Pixel(25.0),
		border: Pixel(2.0),
	};

	pub(crate) fn skill_container_bundle() -> impl Bundle {
		NodeBundle {
			style: Style {
				margin: UiRect::all(Val::from(Self::SKILL_ICON_MARGIN)),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn skill_button_bundle(
		icon: Option<Handle<Image>>,
	) -> impl SkillButtonBundle + Bundle {
		let style = Style {
			width: Val::from(Self::SKILL_BUTTON_DIMENSIONS.width),
			height: Val::from(Self::SKILL_BUTTON_DIMENSIONS.height),
			border: UiRect::from(Self::SKILL_BUTTON_DIMENSIONS.border),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		};

		match icon {
			None => ButtonBundle { style, ..default() },
			Some(icon) => ButtonBundle {
				style,
				background_color: DEFAULT_PANEL_COLORS.filled.into(),
				image: UiImage::new(icon),
				..default()
			},
		}
	}

	pub(crate) fn skill_key_button_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				top: Val::from(Self::MODIFY_BUTTON_OFFSET),
				left: Val::from(Self::MODIFY_BUTTON_OFFSET),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn row_symbol_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				bottom: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.height_inner() / 2. - Self::SYMBOL_WIDTH / 2.,
				),
				left: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.width_inner() / 2. - Self::SYMBOL_WIDTH / 2.,
				),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn column_symbol_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				left: Val::from(
					ComboOverview::SKILL_BUTTON_DIMENSIONS.width_inner() / 2.
						- ComboOverview::SYMBOL_WIDTH / 2.,
				),
				bottom: Val::ZERO,
				..default()
			},
			..default()
		}
	}

	pub(crate) fn delete_button_offset_container() -> impl Bundle {
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

	pub(crate) fn append_button_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				bottom: Val::from(Self::MODIFY_BUTTON_OFFSET),
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

	pub(crate) fn skill_key_bundle() -> impl Bundle {
		NodeBundle {
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

	pub(crate) fn row_line_bundle() -> impl Bundle {
		NodeBundle {
			style: Style {
				width: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.width_outer() + Self::SKILL_ICON_MARGIN * 2.,
				),
				height: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.height_outer() + Self::SKILL_ICON_MARGIN * 2.,
				),
				border: UiRect::bottom(Val::from(Self::SYMBOL_WIDTH)),
				..default()
			},
			border_color: DEFAULT_PANEL_COLORS.filled.into(),
			..default()
		}
	}

	pub(crate) fn column_line_bundle() -> impl Bundle {
		NodeBundle {
			style: Style {
				width: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.width_outer()),
				height: Val::from(
					ComboOverview::SKILL_BUTTON_DIMENSIONS.height_outer() * 1.5
						+ ComboOverview::SKILL_ICON_MARGIN * 3.,
				),
				border: UiRect::left(Val::from(ComboOverview::SYMBOL_WIDTH)),
				..default()
			},
			border_color: DEFAULT_PANEL_COLORS.filled.into(),
			..default()
		}
	}

	pub(crate) fn row_corner_bundle() -> impl Bundle {
		NodeBundle {
			style: Style {
				width: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.width + ComboOverview::SKILL_ICON_MARGIN * 3.,
				),
				height: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.height + ComboOverview::SKILL_ICON_MARGIN * 3.,
				),
				border: UiRect {
					left: Val::from(Self::SYMBOL_WIDTH),
					bottom: Val::from(Self::SYMBOL_WIDTH),
					..default()
				},
				..default()
			},
			border_color: DEFAULT_PANEL_COLORS.filled.into(),
			..default()
		}
	}

	pub(crate) fn skill_key_text(key: SlotKey) -> impl Bundle {
		KeyCodeTextInsertCommandBundle::new(
			key,
			TextStyle {
				font_size: Self::BUTTON_FONT_SIZE,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}

	pub(crate) fn modify_button_text(key: &str) -> impl Bundle {
		TextBundle::from_section(
			key,
			TextStyle {
				font_size: Self::BUTTON_FONT_SIZE,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}
}

impl UpdateCombosView for ComboOverview {
	fn update_combos_view(&mut self, combos: ComboTreeLayout) {
		self.layout = combos
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
			let mut z_index = 0;
			for combo in &combo_overview.layout {
				add_combo(parent, combo, z_index);
				z_index -= 1;
			}
		});
}

fn add_combo(parent: &mut ChildBuilder, combo: &[ComboTreeElement], local_z: i32) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				margin: UiRect::top(Val::from(ComboOverview::SKILL_ICON_MARGIN)),
				..default()
			},
			z_index: ZIndex::Local(local_z),
			..default()
		})
		.with_children(|parent| {
			for element in combo {
				match element {
					ComboTreeElement::Node { key_path, skill } => {
						add_skill(
							parent,
							key_path,
							skill,
							&[add_key, add_append_button, add_delete_button],
							&[add_horizontal_background_line],
						);
					}
					ComboTreeElement::Leaf { key_path, skill } => {
						add_skill(
							parent,
							key_path,
							skill,
							&[add_key, add_append_button, add_delete_button],
							&[],
						);
					}
					ComboTreeElement::Symbol(Symbol::Empty) => {
						add_empty(parent, &[]);
					}
					ComboTreeElement::Symbol(Symbol::Root) => {
						add_empty(parent, &[add_horizontal_background_line]);
					}
					ComboTreeElement::Symbol(Symbol::Line) => {
						add_empty(parent, &[add_vertical_background_line]);
					}
					ComboTreeElement::Symbol(Symbol::Corner) => {
						add_empty(parent, &[add_background_corner]);
					}
				}
			}
		});
}

fn add_empty(parent: &mut ChildBuilder, add_backs: &[fn(&mut ChildBuilder)]) {
	parent
		.spawn(ComboOverview::skill_container_bundle())
		.with_children(|parent| {
			for add_back in add_backs {
				add_back(parent);
			}
			parent.spawn(ComboOverview::skill_button_bundle(None));
		});
}

fn add_skill(
	parent: &mut ChildBuilder,
	key_path: &[SlotKey],
	skill: &Skill,
	add_fronts: &[fn(&[SlotKey], &Skill, &mut ChildBuilder)],
	add_backs: &[fn(&mut ChildBuilder)],
) {
	parent
		.spawn(ComboOverview::skill_container_bundle())
		.with_children(|parent| {
			let icon = skill.icon.clone();
			let button = SkillButton::<DropdownTrigger>::new(skill.clone(), key_path.to_vec());

			for add_back in add_backs {
				add_back(parent);
			}
			parent
				.spawn((
					ComboOverview::skill_button_bundle(icon).with(button),
					SkillSelectDropdownInsertCommand::<SlotKey, Vertical>::new(key_path.to_vec()),
				))
				.with_children(|parent| {
					for add_front in add_fronts {
						add_front(key_path, skill, parent);
					}
				});
		});
}

fn add_vertical_background_line(parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::column_symbol_offset_container())
		.with_children(|parent| {
			parent.spawn(ComboOverview::column_line_bundle());
		});
}

fn add_horizontal_background_line(parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::row_symbol_offset_container())
		.with_children(|parent| {
			parent.spawn(ComboOverview::row_line_bundle());
		});
}

fn add_background_corner(parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::row_symbol_offset_container())
		.with_children(|parent| {
			parent.spawn(ComboOverview::row_corner_bundle());
		});
}

fn add_key(key_path: &[SlotKey], _: &Skill, parent: &mut ChildBuilder) {
	let Some(skill_key) = key_path.last() else {
		return;
	};

	parent
		.spawn(ComboOverview::skill_key_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn(ComboOverview::skill_key_bundle())
				.with_children(|parent| {
					parent.spawn(ComboOverview::skill_key_text(*skill_key));
				});
		});
}

fn add_append_button(key_path: &[SlotKey], _: &Skill, parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::append_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::modify_button_bundle(),
					KeySelectDropdownInsertCommand {
						extra: AppendSkillCommand,
						key_path: key_path.to_vec(),
					},
				))
				.with_children(|parent| {
					parent.spawn(ComboOverview::modify_button_text("+"));
				});
		});
}

fn add_delete_button(key_path: &[SlotKey], _: &Skill, parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::delete_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::modify_button_bundle(),
					DeleteSkill {
						key_path: key_path.to_vec(),
					},
				))
				.with_children(|parent| {
					parent.spawn(ComboOverview::modify_button_text("x"));
				});
		});
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::combo_tree_layout::ComboTreeElement;
	use bevy::asset::{Asset, AssetId};
	use common::{components::Side, simple_init, traits::mock::Mock};
	use mockall::{mock, predicate::eq};
	use skills::skills::Skill;
	use uuid::Uuid;

	#[test]
	fn update_combos() {
		let combos = vec![vec![ComboTreeElement::Leaf {
			skill: Skill {
				name: "my skill".to_owned(),
				icon: Some(Handle::Weak(AssetId::Uuid {
					uuid: Uuid::new_v4(),
				})),
				..default()
			},
			key_path: vec![SlotKey::Hand(Side::Main)],
		}]];
		let mut combo_overview = ComboOverview::default();
		combo_overview.update_combos_view(combos.clone());

		assert_eq!(combos, combo_overview.layout)
	}

	mock! {
		_Server {}
		impl LoadAsset for _Server {
			fn load_asset<TAsset: Asset>(&mut self, path: Path) -> Handle<TAsset>;
		}
	}

	simple_init!(Mock_Server);

	#[test]
	fn load_ui_with_asset_handle() {
		let handle = Handle::<Image>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut server = Mock_Server::new_mock(|mock| {
			mock.expect_load_asset().return_const(handle.clone());
		});
		let combos = ComboOverview::load_ui(&mut server);

		assert_eq!(
			ComboOverview {
				new_skill_icon: handle,
				..default()
			},
			combos
		);
	}

	#[test]
	fn load_ui_with_asset_of_correct_path() {
		let mut server = Mock_Server::new_mock(|mock| {
			mock.expect_load_asset::<Image>()
				.times(1)
				.with(eq(Path::from("icons/empty.png")))
				.return_const(Handle::default());
		});
		_ = ComboOverview::load_ui(&mut server);
	}
}
