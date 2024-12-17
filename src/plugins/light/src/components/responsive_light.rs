use super::{
	responsive_light_change::ResponsiveLightChange,
	responsive_light_trigger::ResponsiveLightTrigger,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollidingEntities, Sensor};
use common::{
	errors::Error,
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::{
		handles_lights::Responsive,
		has_collisions::HasCollisions,
		prefab::{GetOrCreateAssets, Prefab},
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};
use std::{ops::Deref, time::Duration};

#[derive(Component, Debug, PartialEq, Clone)]
pub struct ResponsiveLight {
	pub model: Entity,
	pub light: Entity,
	pub range: Units,
	pub light_on_material: Handle<StandardMaterial>,
	pub light_off_material: Handle<StandardMaterial>,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
}

impl ResponsiveLight {
	pub(crate) fn detect_change<TColliderCollection: HasCollisions + Component>(
		mut commands: Commands,
		responsive_lights: Query<
			(Entity, &ResponsiveLight, &TColliderCollection),
			Changed<TColliderCollection>,
		>,
		triggers: Query<&ResponsiveLightTrigger>,
	) {
		for (id, responsive, collisions) in &responsive_lights {
			let change_light = get_change(responsive, collisions, &triggers);
			commands.try_insert_on(id, change_light);
		}
	}

	pub(crate) fn apply_change<TTime: Sync + Send + 'static + Default>(
		mut commands: Commands,
		mut lights: Query<&mut PointLight>,
		changes: Query<(Entity, &ResponsiveLightChange)>,
		time: Res<Time<TTime>>,
	) {
		let delta = time.delta();
		for (id, change) in &changes {
			let state = apply_change(&mut commands, &mut lights, change, delta);
			remove_change_component(&mut commands, state, id);
		}
	}
}

impl From<Responsive> for ResponsiveLight {
	fn from(data: Responsive) -> Self {
		ResponsiveLight {
			model: data.model,
			light: data.light,
			range: data.range,
			light_on_material: data.light_on_material,
			light_off_material: data.light_off_material,
			max: data.max,
			change: data.change,
		}
	}
}

impl Prefab<()> for ResponsiveLight {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		entity.try_insert((
			TransformBundle::default(),
			Collider::ball(*self.range.deref()),
			Sensor,
			ActiveEvents::COLLISION_EVENTS,
			CollidingEntities::default(),
		));

		Ok(())
	}
}

#[derive(PartialEq)]
enum State {
	Done,
	Busy,
}

fn apply_change(
	commands: &mut Commands,
	lights: &mut Query<&mut PointLight>,
	change: &ResponsiveLightChange,
	delta: Duration,
) -> State {
	match change {
		ResponsiveLightChange::Increase(light) => increase(commands, lights, light, delta),
		ResponsiveLightChange::Decrease(light) => decrease(commands, lights, light, delta),
	}
}

fn remove_change_component(commands: &mut Commands, state: State, id: Entity) {
	if state != State::Done {
		return;
	}
	commands.try_remove_from::<ResponsiveLightChange>(id);
}

fn increase(
	commands: &mut Commands,
	lights: &mut Query<&mut PointLight>,
	light: &ResponsiveLight,
	delta: Duration,
) -> State {
	let Ok(mut target_light) = lights.get_mut(light.light) else {
		return State::Busy;
	};

	if target_light.intensity == 0. {
		commands.try_insert_on(light.light, Visibility::Visible);
		commands.try_insert_on(light.model, light.light_on_material.clone());
	}

	let change = *light.change.deref() * delta.as_secs_f32();
	let max = *light.max.deref();
	if max - target_light.intensity > change {
		target_light.intensity += change;
		return State::Busy;
	}

	target_light.intensity = max;

	State::Done
}

fn decrease(
	commands: &mut Commands,
	lights: &mut Query<&mut PointLight>,
	light: &ResponsiveLight,
	delta: Duration,
) -> State {
	let Ok(mut target_light) = lights.get_mut(light.light) else {
		return State::Busy;
	};

	let change = *light.change.deref() * delta.as_secs_f32();
	if change < target_light.intensity {
		target_light.intensity -= change;
		return State::Busy;
	}

	target_light.intensity = 0.;
	commands.try_insert_on(light.model, light.light_off_material.clone());
	commands.try_insert_on(light.light, Visibility::Hidden);

	State::Done
}

fn get_change<TColliderCollection: HasCollisions>(
	responsive: &ResponsiveLight,
	collisions: &TColliderCollection,
	triggers: &Query<&ResponsiveLightTrigger>,
) -> ResponsiveLightChange {
	if collisions.collisions().any(|e| triggers.contains(e)) {
		return ResponsiveLightChange::Increase(responsive.clone());
	}

	ResponsiveLightChange::Decrease(responsive.clone())
}

#[cfg(test)]
mod test_detect_change {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::{Intensity, IntensityChangePerSecond, Units},
		traits::clamp_zero_positive::ClampZeroPositive,
	};
	use uuid::Uuid;

	#[derive(Component)]
	struct _Collisions(Vec<Entity>);

	impl HasCollisions for _Collisions {
		fn collisions(&self) -> impl Iterator<Item = Entity> + '_ {
			self.0.iter().cloned()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ResponsiveLight::detect_change::<_Collisions>);

		app
	}

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[test]
	fn apply_on() {
		let mut app = setup();
		let light_on_material = new_handle();
		let trigger = app.world_mut().spawn(ResponsiveLightTrigger).id();
		let model = app.world_mut().spawn_empty().id();
		let light = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: light_on_material.clone(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};

		let entity = app
			.world_mut()
			.spawn((responsive.clone(), _Collisions(vec![trigger])))
			.id();

		app.update();

		assert_eq!(
			Some(&ResponsiveLightChange::Increase(responsive)),
			app.world().entity(entity).get::<ResponsiveLightChange>(),
		)
	}

	#[test]
	fn apply_off() {
		let mut app = setup();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let light = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: light_off_material.clone(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};

		let entity = app
			.world_mut()
			.spawn((responsive.clone(), _Collisions(vec![])))
			.id();

		app.update();

		assert_eq!(
			Some(&ResponsiveLightChange::Decrease(responsive)),
			app.world().entity(entity).get::<ResponsiveLightChange>(),
		)
	}
}

#[cfg(test)]
mod test_apply_change {
	use super::*;
	use common::{
		test_tools::utils::{SingleThreadedApp, TickTime},
		tools::{Intensity, IntensityChangePerSecond, Units},
		traits::clamp_zero_positive::ClampZeroPositive,
	};
	use std::time::Duration;
	use uuid::Uuid;

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Time<Real>>();
		app.add_systems(Update, ResponsiveLight::apply_change::<Real>);

		app
	}

	#[test]
	fn increase_light_intensity() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(53., light.intensity);
	}

	#[test]
	fn increase_light_intensity_scaled_by_delta() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive));

		app.tick_time(Duration::from_millis(100));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(43.1, light.intensity);
	}

	#[test]
	fn increase_light_intensity_clamped_at_max() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(200.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(100., light.intensity);
	}

	#[test]
	fn insert_light_visibility_on_increase() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 0.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(Some(&Visibility::Visible), light.get::<Visibility>());
	}

	#[test]
	fn do_not_insert_light_visibility_on_increase_when_intensity_not_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 1.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(None, light.get::<Visibility>());
	}

	#[test]
	fn set_light_on_material_when_increasing() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 0.,
				..default()
			})
			.id();
		let light_on_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: light_on_material.clone(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(
			Some(&light_on_material),
			model.get::<Handle<StandardMaterial>>()
		);
	}

	#[test]
	fn do_not_set_light_on_material_when_intensity_not_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 1.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(None, model.get::<Handle<StandardMaterial>>());
	}

	#[test]
	fn remove_change_light_when_reached_max() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(58.),
		};
		let responsive = app
			.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive = app.world().entity(responsive);

		assert_eq!(None, responsive.get::<ResponsiveLightChange>());
	}

	#[test]
	fn do_not_remove_change_light_when_not_reached_max() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(57.),
		};
		let responsive_entity = app
			.world_mut()
			.spawn(ResponsiveLightChange::Increase(responsive.clone()))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive_entity = app.world().entity(responsive_entity);

		assert_eq!(
			Some(&ResponsiveLightChange::Increase(responsive)),
			responsive_entity.get::<ResponsiveLightChange>()
		);
	}

	#[test]
	fn decrease_light_intensity() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(31., light.intensity);
	}

	#[test]
	fn decrease_light_intensity_by_delta() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(11.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_millis(100));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(40.9, light.intensity);
	}

	#[test]
	fn set_light_off_material_when_decreasing_to_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: light_off_material.clone(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(42.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(
			Some(&light_off_material),
			model.get::<Handle<StandardMaterial>>()
		);
	}

	#[test]
	fn do_not_set_light_off_material_when_decreasing_above_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: light_off_material.clone(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(41.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(None, model.get::<Handle<StandardMaterial>>());
	}

	#[test]
	fn set_light_off_material_when_decreasing_to_below_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let light_off_material = new_handle();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: light_off_material.clone(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(43.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let model = app.world().entity(model);

		assert_eq!(
			Some(&light_off_material),
			model.get::<Handle<StandardMaterial>>()
		);
	}

	#[test]
	fn decrease_light_intensity_clamped_at_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(43.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light).get::<PointLight>().unwrap();

		assert_eq!(0., light.intensity);
	}

	#[test]
	fn remove_change_light_when_reached_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(42.),
		};
		let responsive = app
			.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive = app.world().entity(responsive);

		assert_eq!(None, responsive.get::<ResponsiveLightChange>());
	}

	#[test]
	fn do_not_remove_change_light_when_not_reached_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(41.),
		};
		let responsive_entity = app
			.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive.clone()))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		let responsive_entity = app.world().entity(responsive_entity);

		assert_eq!(
			Some(&ResponsiveLightChange::Decrease(responsive)),
			responsive_entity.get::<ResponsiveLightChange>()
		);
	}

	#[test]
	fn insert_light_hidden_when_reaching_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(42.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(Some(&Visibility::Hidden), light.get::<Visibility>());
	}

	#[test]
	fn do_not_insert_light_hidden_when_not_reaching_zero() {
		let mut app = setup();
		let light = app
			.world_mut()
			.spawn(PointLight {
				intensity: 42.,
				..default()
			})
			.id();
		let model = app.world_mut().spawn_empty().id();
		let responsive = ResponsiveLight {
			model,
			light,
			range: Units::new(0.),
			light_on_material: new_handle(),
			light_off_material: new_handle(),
			max: Intensity::new(100.),
			change: IntensityChangePerSecond::new(41.),
		};
		app.world_mut()
			.spawn(ResponsiveLightChange::Decrease(responsive));

		app.tick_time(Duration::from_secs(1));
		app.update();

		let light = app.world().entity(light);

		assert_eq!(None, light.get::<Visibility>());
	}
}