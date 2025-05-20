use bevy::prelude::*;

#[derive(Component, Default, Debug, PartialEq)]
#[require(BackgroundColor = Self::gray())]
pub(crate) struct MenuBackground {
	overrides: Overrides,
}

impl MenuBackground {
	fn gray() -> BackgroundColor {
		BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5))
	}

	fn full_screen() -> Node {
		Node {
			width: Val::Vw(100.0),
			height: Val::Vh(100.0),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		}
	}

	pub(crate) fn node(&self) -> Node {
		let mut node = Self::full_screen();
		self.overrides.override_values(&mut node);
		node
	}
}

pub(crate) trait WithOverride<T> {
	fn with(self, override_value: T) -> Self;
}

macro_rules! impl_with_override_trait {
	($ty:ty, $field:ident) => {
		impl WithOverride<$ty> for MenuBackground {
			fn with(mut self, $field: $ty) -> Self {
				self.overrides.$field = Some($field);
				self
			}
		}
	};
}

impl_with_override_trait!(AlignItems, align_items);
impl_with_override_trait!(JustifyContent, justify_content);
impl_with_override_trait!(FlexDirection, flex_direction);

#[derive(Default, Debug, PartialEq)]
struct Overrides {
	align_items: Option<AlignItems>,
	justify_content: Option<JustifyContent>,
	flex_direction: Option<FlexDirection>,
}

macro_rules! impl_override_values {
	($self:expr, $node:expr, $field:ident) => {
		if let Some($field) = $self.$field {
			$node.$field = $field;
		}
	};
}

impl Overrides {
	fn override_values(&self, node: &mut Node) {
		impl_override_values!(self, node, align_items);
		impl_override_values!(self, node, justify_content);
		impl_override_values!(self, node, flex_direction);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fmt::Debug;
	use test_case::test_case;

	#[test_case(AlignItems::End, |node| node.align_items; "align items")]
	#[test_case(JustifyContent::End, |node| node.justify_content; "justify content")]
	#[test_case(FlexDirection::RowReverse, |node| node.flex_direction; "flex direction")]
	fn override_align_items<T>(value: T, get_item: fn(&Node) -> T)
	where
		T: PartialEq + Debug + Copy,
		MenuBackground: WithOverride<T>,
	{
		let background = MenuBackground::default().with(value);
		let node = background.node();

		assert_eq!(value, get_item(&node));
	}
}
