use super::{
	key_code_text_insert_command::KeyCodeTextInsertCommand,
	key_select_dropdown_command::{AppendSkillCommand, KeySelectDropdownCommand},
	skill_button::{DropdownTrigger, SkillButton, Vertical},
	DeleteSkill,
	SkillSelectDropdownInsertCommand,
};
use crate::{
	tools::{Dimensions, Pixel},
	traits::{
		colors::DEFAULT_PANEL_COLORS,
		combo_tree_layout::{ComboTreeElement, ComboTreeLayout, Symbol},
		insert_ui_content::InsertUiContent,
		LoadUi,
		UpdateCombosView,
	},
};
use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{
		accessors::get::{GetFieldRef, GetterRef},
		load_asset::{LoadAsset, Path},
		thread_safe::ThreadSafe,
	},
};

#[derive(Component, Debug, PartialEq)]
#[require(Node(full_screen), BackgroundColor(gray), Name(|| "Combo Overview"))]
pub(crate) struct ComboOverview<TSkill>
where
	TSkill: ThreadSafe,
{
	new_skill_icon: Handle<Image>,
	layout: ComboTreeLayout<TSkill>,
}

impl<TSKill> Default for ComboOverview<TSKill>
where
	TSKill: ThreadSafe,
{
	fn default() -> Self {
		Self {
			new_skill_icon: Default::default(),
			layout: Default::default(),
		}
	}
}

fn full_screen() -> Node {
	Node {
		width: Val::Vw(100.),
		height: Val::Vh(100.),
		flex_direction: FlexDirection::Column,
		..default()
	}
}

fn gray() -> BackgroundColor {
	BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5))
}

impl<TAssetServer, TSKill> LoadUi<TAssetServer> for ComboOverview<TSKill>
where
	TAssetServer: LoadAsset,
	TSKill: ThreadSafe,
{
	fn load_ui(images: &mut TAssetServer) -> Self {
		ComboOverview {
			new_skill_icon: images.load_asset(Path::from("icons/empty.png")),
			..default()
		}
	}
}

impl ComboOverview<()> {
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

	pub(crate) fn skill_node() -> Node {
		Node {
			margin: UiRect::all(Val::from(Self::SKILL_ICON_MARGIN)),
			..default()
		}
	}

	pub(crate) fn skill_button<T>(icon: T) -> (Button, Node, ImageNode, BackgroundColor)
	where
		SkillButtonIcon: From<T>,
	{
		let node = Node {
			width: Val::from(Self::SKILL_BUTTON_DIMENSIONS.width),
			height: Val::from(Self::SKILL_BUTTON_DIMENSIONS.height),
			border: UiRect::from(Self::SKILL_BUTTON_DIMENSIONS.border),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		};
		let (image, background_color) = match SkillButtonIcon::from(icon) {
			SkillButtonIcon::Icon(Some(icon)) => (
				ImageNode::new(icon),
				BackgroundColor(DEFAULT_PANEL_COLORS.filled),
			),
			SkillButtonIcon::Icon(None) => (
				ImageNode::default(),
				BackgroundColor(DEFAULT_PANEL_COLORS.filled),
			),
			SkillButtonIcon::Transparent => (default(), BackgroundColor(Color::NONE)),
		};

		(Button, node, image, background_color)
	}

	pub(crate) fn skill_key_button_offset_node() -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(Self::MODIFY_BUTTON_OFFSET),
			left: Val::from(Self::MODIFY_BUTTON_OFFSET),
			..default()
		}
	}

	pub(crate) fn row_symbol_offset_node() -> Node {
		Node {
			position_type: PositionType::Absolute,
			bottom: Val::from(
				Self::SKILL_BUTTON_DIMENSIONS.height_inner() / 2. - Self::SYMBOL_WIDTH / 2.,
			),
			left: Val::from(
				Self::SKILL_BUTTON_DIMENSIONS.width_inner() / 2. - Self::SYMBOL_WIDTH / 2.,
			),
			..default()
		}
	}

	pub(crate) fn column_symbol_offset_node() -> Node {
		Node {
			position_type: PositionType::Absolute,
			left: Val::from(
				ComboOverview::SKILL_BUTTON_DIMENSIONS.width_inner() / 2.
					- ComboOverview::SYMBOL_WIDTH / 2.,
			),
			bottom: Val::ZERO,
			..default()
		}
	}

	pub(crate) fn delete_button_offset_node() -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(Self::MODIFY_BUTTON_OFFSET),
			right: Val::from(Self::MODIFY_BUTTON_OFFSET),
			..default()
		}
	}

	pub(crate) fn append_button_offset_node() -> Node {
		Node {
			position_type: PositionType::Absolute,
			bottom: Val::from(Self::MODIFY_BUTTON_OFFSET),
			right: Val::from(Self::MODIFY_BUTTON_OFFSET),
			..default()
		}
	}

	pub(crate) fn skill_key_button() -> (Button, Node, BackgroundColor, BorderColor) {
		(
			Button,
			Node {
				width: Val::from(Self::KEY_BUTTON_DIMENSIONS.width),
				height: Val::from(Self::KEY_BUTTON_DIMENSIONS.height),
				border: UiRect::from(Self::KEY_BUTTON_DIMENSIONS.border),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			DEFAULT_PANEL_COLORS.filled.into(),
			DEFAULT_PANEL_COLORS.text.into(),
		)
	}

	pub(crate) fn modify_button() -> (Button, Node, BackgroundColor, BorderColor) {
		(
			Button,
			Node {
				width: Val::from(Self::MODIFY_BUTTON_DIMENSIONS.width),
				height: Val::from(Self::MODIFY_BUTTON_DIMENSIONS.height),
				border: UiRect::from(Self::MODIFY_BUTTON_DIMENSIONS.border),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			DEFAULT_PANEL_COLORS.filled.into(),
			DEFAULT_PANEL_COLORS.text.into(),
		)
	}

	pub(crate) fn row_line() -> (Node, BorderColor) {
		(
			Node {
				width: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.width_outer() + Self::SKILL_ICON_MARGIN * 2.,
				),
				height: Val::from(
					Self::SKILL_BUTTON_DIMENSIONS.height_outer() + Self::SKILL_ICON_MARGIN * 2.,
				),
				border: UiRect::bottom(Val::from(Self::SYMBOL_WIDTH)),
				..default()
			},
			DEFAULT_PANEL_COLORS.filled.into(),
		)
	}

	pub(crate) fn column_line() -> (Node, BorderColor) {
		(
			Node {
				width: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.width_outer()),
				height: Val::from(
					ComboOverview::SKILL_BUTTON_DIMENSIONS.height_outer() * 1.5
						+ ComboOverview::SKILL_ICON_MARGIN * 3.,
				),
				border: UiRect::left(Val::from(ComboOverview::SYMBOL_WIDTH)),
				..default()
			},
			DEFAULT_PANEL_COLORS.filled.into(),
		)
	}

	pub(crate) fn row_corner() -> (Node, BorderColor) {
		(
			Node {
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
			DEFAULT_PANEL_COLORS.filled.into(),
		)
	}

	pub(crate) fn skill_key_text(key: SlotKey) -> KeyCodeTextInsertCommand<SlotKey> {
		KeyCodeTextInsertCommand {
			key,
			font: TextFont {
				font_size: Self::BUTTON_FONT_SIZE,
				..default()
			},
			color: TextColor(DEFAULT_PANEL_COLORS.text),
			..default()
		}
	}

	pub(crate) fn modify_button_text(key: &str) -> (Text, TextFont, TextColor) {
		(
			Text::new(key),
			TextFont {
				font_size: Self::BUTTON_FONT_SIZE,
				..default()
			},
			TextColor(DEFAULT_PANEL_COLORS.text),
		)
	}
}

pub(crate) enum SkillButtonIcon {
	Icon(Option<Handle<Image>>),
	Transparent,
}

impl From<Handle<Image>> for SkillButtonIcon {
	fn from(icon: Handle<Image>) -> Self {
		SkillButtonIcon::Icon(Some(icon))
	}
}

impl From<Option<Handle<Image>>> for SkillButtonIcon {
	fn from(icon: Option<Handle<Image>>) -> Self {
		SkillButtonIcon::Icon(icon)
	}
}

impl<TSkill> UpdateCombosView<TSkill> for ComboOverview<TSkill>
where
	TSkill: ThreadSafe,
{
	fn update_combos_view(&mut self, combos: ComboTreeLayout<TSkill>) {
		self.layout = combos
	}
}

impl<TSkill> InsertUiContent for ComboOverview<TSkill>
where
	TSkill: GetterRef<Option<Handle<Image>>> + Clone + ThreadSafe,
{
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		add_title(parent, "Combos");
		if self.layout.is_empty() {
			add_empty_combo(parent, &self.new_skill_icon);
		} else {
			add_combo_list(parent, self);
		}
	}
}

fn add_title(parent: &mut ChildBuilder, title: &str) {
	parent
		.spawn(Node {
			margin: UiRect::all(Val::Px(10.)),
			..default()
		})
		.with_children(|parent| {
			parent.spawn((
				Text::new(title),
				TextFont {
					font_size: 40.,
					..default()
				},
				TextColor(DEFAULT_PANEL_COLORS.text),
			));
		});
}

fn add_empty_combo(parent: &mut ChildBuilder, icon: &Handle<Image>) {
	parent
		.spawn(Node {
			flex_direction: FlexDirection::Column,
			..default()
		})
		.with_children(|parent| {
			parent
				.spawn(Node {
					flex_direction: FlexDirection::Row,
					margin: UiRect::top(Val::from(ComboOverview::SKILL_ICON_MARGIN)),
					..default()
				})
				.with_children(|parent| {
					add_combo_starter(parent, icon, &[add_append_button], &[]);
				});
		});
}

fn add_combo_starter(
	parent: &mut ChildBuilder,
	icon: &Handle<Image>,
	add_fronts: &[fn(&[SlotKey], &mut ChildBuilder)],
	add_backs: &[fn(&mut ChildBuilder)],
) {
	parent
		.spawn(ComboOverview::skill_node())
		.with_children(|parent| {
			for add_back in add_backs {
				add_back(parent);
			}
			parent
				.spawn(ComboOverview::skill_button(icon.clone()))
				.with_children(|parent| {
					for add_front in add_fronts {
						add_front(&[], parent);
					}
				});
		});
}

fn add_combo_list<TSkill>(parent: &mut ChildBuilder, combo_overview: &ComboOverview<TSkill>)
where
	TSkill: GetterRef<Option<Handle<Image>>> + Clone + ThreadSafe,
{
	parent
		.spawn(Node {
			flex_direction: FlexDirection::Column,
			..default()
		})
		.with_children(|parent| {
			let mut z_index = 0;
			for combo in &combo_overview.layout {
				add_combo(parent, combo, z_index, &combo_overview.new_skill_icon);
				z_index -= 1;
			}
		});
}

fn add_combo<TSkill>(
	parent: &mut ChildBuilder,
	combo: &[ComboTreeElement<TSkill>],
	local_z: i32,
	new_skill_icon: &Handle<Image>,
) where
	TSkill: GetterRef<Option<Handle<Image>>> + Clone + ThreadSafe,
{
	parent
		.spawn((
			#[cfg(debug_assertions)]
			Name::from("Combo Row"),
			Node {
				flex_direction: FlexDirection::Row,
				margin: UiRect::top(Val::from(ComboOverview::SKILL_ICON_MARGIN)),
				..default()
			},
			ZIndex(local_z),
		))
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
						add_combo_starter(
							parent,
							new_skill_icon,
							&[add_append_button],
							&[add_horizontal_background_line],
						);
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
		.spawn((
			#[cfg(debug_assertions)]
			Name::from("Empty"),
			ComboOverview::skill_node(),
		))
		.with_children(|parent| {
			for add_back in add_backs {
				add_back(parent);
			}
			parent.spawn((
				Name::from("Empty Button"),
				ComboOverview::skill_button(SkillButtonIcon::Transparent),
			));
		});
}

fn add_skill<TSkill>(
	parent: &mut ChildBuilder,
	key_path: &[SlotKey],
	skill: &TSkill,
	add_fronts: &[fn(&[SlotKey], &mut ChildBuilder)],
	add_backs: &[fn(&mut ChildBuilder)],
) where
	TSkill: GetterRef<Option<Handle<Image>>> + Clone + ThreadSafe,
{
	let icon = Option::<Handle<Image>>::get_field_ref(skill);
	parent
		.spawn((
			#[cfg(debug_assertions)]
			Name::from("Skill"),
			ComboOverview::skill_node(),
		))
		.with_children(|parent| {
			let button =
				SkillButton::<DropdownTrigger, TSkill>::new(skill.clone(), key_path.to_vec());

			for add_back in add_backs {
				add_back(parent);
			}
			parent
				.spawn((
					button,
					ComboOverview::skill_button(icon.clone()),
					SkillSelectDropdownInsertCommand::<SlotKey, Vertical>::new(key_path.to_vec()),
				))
				.with_children(|parent| {
					for add_front in add_fronts {
						add_front(key_path, parent);
					}
				});
		});
}

fn add_vertical_background_line(parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::column_symbol_offset_node())
		.with_children(|parent| {
			parent.spawn(ComboOverview::column_line());
		});
}

fn add_horizontal_background_line(parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::row_symbol_offset_node())
		.with_children(|parent| {
			parent.spawn(ComboOverview::row_line());
		});
}

fn add_background_corner(parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::row_symbol_offset_node())
		.with_children(|parent| {
			parent.spawn(ComboOverview::row_corner());
		});
}

fn add_key(key_path: &[SlotKey], parent: &mut ChildBuilder) {
	let Some(skill_key) = key_path.last() else {
		return;
	};

	parent
		.spawn(ComboOverview::skill_key_button_offset_node())
		.with_children(|parent| {
			parent
				.spawn(ComboOverview::skill_key_button())
				.with_children(|parent| {
					parent.spawn(ComboOverview::skill_key_text(*skill_key));
				});
		});
}

fn add_append_button(key_path: &[SlotKey], parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::append_button_offset_node())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::modify_button(),
					KeySelectDropdownCommand {
						extra: AppendSkillCommand,
						key_path: key_path.to_vec(),
					},
				))
				.with_children(|parent| {
					parent.spawn(ComboOverview::modify_button_text("+"));
				});
		});
}

fn add_delete_button(key_path: &[SlotKey], parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::delete_button_offset_node())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::modify_button(),
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
	use bevy::asset::{Asset, AssetId, AssetPath};
	use common::{simple_init, tools::slot_key::Side, traits::mock::Mock};
	use mockall::{mock, predicate::eq};
	use uuid::Uuid;

	#[derive(Debug, PartialEq, Clone)]
	struct _Skill;

	#[test]
	fn update_combos() {
		let combos = vec![vec![ComboTreeElement::Leaf {
			skill: _Skill,
			key_path: vec![SlotKey::BottomHand(Side::Right)],
		}]];
		let mut combo_overview = ComboOverview::default();
		combo_overview.update_combos_view(combos.clone());

		assert_eq!(combos, combo_overview.layout)
	}

	mock! {
		_Server {}
		impl LoadAsset for _Server {
			fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
			where
				TAsset: Asset,
				TPath: Into<AssetPath<'static>> + 'static;
		}
	}

	simple_init!(Mock_Server);

	#[test]
	fn load_ui_with_asset_handle() {
		let handle = Handle::<Image>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let mut server = Mock_Server::new_mock(|mock| {
			mock.expect_load_asset::<Image, Path>()
				.return_const(handle.clone());
		});
		let combos = ComboOverview::<_Skill>::load_ui(&mut server);

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
			mock.expect_load_asset::<Image, Path>()
				.times(1)
				.with(eq(Path::from("icons/empty.png")))
				.return_const(Handle::default());
		});
		_ = ComboOverview::<_Skill>::load_ui(&mut server);
	}
}
