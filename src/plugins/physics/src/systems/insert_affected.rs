use bevy::prelude::*;
use common::{
	tools::attribute::AttributeOnSpawn,
	traits::accessors::get::{GetProperty, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> InsertAffected for T where T: AffectedComponent {}

pub(crate) trait InsertAffected: AffectedComponent {
	fn insert_from<TDefaultAttributes>(
		mut commands: ZyheedaCommands,
		default_attributes: Query<(Entity, &TDefaultAttributes), Without<Self>>,
	) where
		TDefaultAttributes: Component + GetProperty<AttributeOnSpawn<Self::TAttribute>>,
	{
		for (entity, default_attribute) in &default_attributes {
			commands.try_apply_on(&entity, |mut e| {
				let attribute = default_attribute.get_property();
				e.try_insert(Self::from(attribute));
			});
		}
	}
}

pub(crate) trait AffectedComponent: Component + From<Self::TAttribute> {
	type TAttribute;
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq)]
	struct _Affected(_Attribute);

	impl From<_Attribute> for _Affected {
		fn from(attribute: _Attribute) -> Self {
			Self(attribute)
		}
	}

	impl AffectedComponent for _Affected {
		type TAttribute = _Attribute;
	}

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	struct _Attribute(&'static str);

	#[derive(Component)]
	struct _DefaultAttribute(_Attribute);

	impl GetProperty<AttributeOnSpawn<_Attribute>> for _DefaultAttribute {
		fn get_property(&self) -> _Attribute {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Affected::insert_from::<_DefaultAttribute>);

		app
	}

	#[test]
	fn insert_when_asset_loaded() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_DefaultAttribute(_Attribute("my attribute")))
			.id();

		app.update();

		assert_eq!(
			Some(&_Affected(_Attribute("my attribute"))),
			app.world().entity(entity).get::<_Affected>(),
		);
	}

	#[test]
	fn do_not_insert_when_affected_already_present() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_DefaultAttribute(_Attribute("new attribute")),
				_Affected(_Attribute("old attribute")),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Affected(_Attribute("old attribute"))),
			app.world().entity(entity).get::<_Affected>(),
		);
	}
}
