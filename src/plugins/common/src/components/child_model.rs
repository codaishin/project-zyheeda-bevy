use crate::{
	components::model::Model,
	errors::Unreachable,
	traits::{
		accessors::get::TryApplyOn,
		prefab::{Prefab, Reapply},
	},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};

#[derive(Component, Debug, PartialEq)]
#[require(Visibility)]
#[component(immutable)]
pub struct ChildModel(pub Model);

impl Prefab<()> for ChildModel {
	type TError = Unreachable;
	type TSystemParam = (
		ZyheedaCommands<'static, 'static>,
		Query<'static, 'static, &'static ChildModelRootFor>,
	);

	const REAPPLY: Reapply = Reapply::Always;

	fn insert_prefab_components(
		&self,
		entity: &mut impl crate::prelude::PrefabEntityCommands,
		params: StaticSystemParam<Self::TSystemParam>,
	) -> Result<(), Self::TError> {
		let (mut commands, roots) = params.into_inner();

		if let Ok(ChildModelRootFor(child)) = roots.get(entity.entity_id()) {
			commands.try_apply_on(child, |c| c.try_despawn());
		}

		commands.spawn((
			ChildOf(entity.entity_id()),
			ChildModelOf(entity.entity_id()),
			self.0.clone(),
		));

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = ChildModelOf, linked_spawn)]
pub struct ChildModelRootFor(Entity);

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = ChildModelRootFor)]
pub struct ChildModelOf(Entity);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::model::Scene, traits::prefab::AddPrefabObserver};
	use testing::{SingleThreadedApp, assert_count};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_prefab_observer::<ChildModel, ()>();

		app
	}

	#[test]
	fn insert_child() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(ChildModel(Model::Scene(Scene::default())))
			.id();

		let mut children = app.world_mut().query::<&ChildOf>();
		let [ChildOf(parent)] = assert_count!(1, children.iter(app.world()));
		assert_eq!(entity, *parent);
	}

	#[test]
	fn insert_child_model() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(ChildModel(Model::Scene(Scene::default())))
			.id();

		let mut children = app.world_mut().query::<(&ChildOf, &Model)>();
		let children = children
			.iter(app.world())
			.filter(|(ChildOf(parent), ..)| *parent == entity);
		let [(.., model)] = assert_count!(1, children);
		assert_eq!(&Model::Scene(Scene::default()), model);
	}

	#[test]
	fn replace_child_model() {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(ChildModel(Model::None));
		entity.insert(ChildModel(Model::Scene(Scene::default())));

		let entity = entity.id();
		let mut children = app.world_mut().query::<(&ChildOf, &Model)>();
		let children = children
			.iter(app.world())
			.filter(|(ChildOf(parent), ..)| *parent == entity);
		let [(.., model)] = assert_count!(1, children);
		assert_eq!(&Model::Scene(Scene::default()), model);
	}

	#[test]
	fn do_not_touch_other_children_on_replace() {
		#[derive(Component)]
		struct _Other;

		let mut app = setup();

		let mut entity = app.world_mut().spawn(ChildModel(Model::None));
		entity.with_child(_Other);
		entity.insert(ChildModel(Model::Scene(Scene::default())));

		let mut others = app.world_mut().query::<&_Other>();
		assert_count!(1, others.iter(app.world()));
	}
}
