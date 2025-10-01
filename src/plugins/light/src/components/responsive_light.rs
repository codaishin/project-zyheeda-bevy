use super::{
	responsive_light_change::ResponsiveLightChange,
	responsive_light_trigger::ResponsiveLightTrigger,
};
use crate::traits::light_components::LightComponent;
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollidingEntities, Sensor};
use common::{
	components::insert_asset::InsertAsset,
	errors::Unreachable,
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::{
		accessors::get::TryApplyOn,
		handles_lights::{Light, Responsive},
		has_collisions::HasCollisions,
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::{any::TypeId, ops::Deref, time::Duration};

#[derive(Component, Debug, Clone)]
#[require(
	Transform,
	Sensor,
	ActiveEvents = ActiveEvents::COLLISION_EVENTS,
	CollidingEntities,
)]
pub struct ResponsiveLight {
	pub light: Light,
	pub range: Units,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
	pub light_on_material: fn() -> StandardMaterial,
	pub marker_on: TypeId,
	pub light_off_material: fn() -> StandardMaterial,
	pub marker_off: TypeId,
}

impl ResponsiveLight {
	pub(crate) fn detect_change<TColliderCollection: HasCollisions + Component>(
		mut commands: ZyheedaCommands,
		responsive_lights: Query<(Entity, &TColliderCollection), Changed<TColliderCollection>>,
		triggers: Query<&ResponsiveLightTrigger>,
	) {
		for (entity, collisions) in &responsive_lights {
			let change_light = get_change(collisions, &triggers);
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(change_light);
			});
		}
	}

	pub(crate) fn apply_change<TTime, TLight>(
		mut commands: Commands,
		mut lights: Query<(Entity, &ResponsiveLightChange, &Self, &mut TLight)>,
		time: Res<Time<TTime>>,
	) where
		TTime: Default + ThreadSafe,
		TLight: LightComponent,
	{
		let delta = time.delta();
		for (entity, change, responsive, mut light) in &mut lights {
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			let state = apply_change(&mut entity, change, responsive, light.as_mut(), delta);
			remove_change_component(&mut entity, state);
		}
	}

	pub(crate) fn for_driver<TDriver>(data: Responsive) -> Self
	where
		TDriver: 'static,
	{
		ResponsiveLight {
			light: data.light,
			range: data.range,
			light_on_material: data.light_on_material,
			marker_on: TypeId::of::<(TDriver, LightOn)>(),
			light_off_material: data.light_off_material,
			marker_off: TypeId::of::<(TDriver, LightOff)>(),
			max: data.max,
			change: data.change,
		}
	}

	pub(crate) fn insert_light(
		mut commands: ZyheedaCommands,
		lights: Query<(Entity, &Self), Added<Self>>,
	) {
		for (entity, responsive) in &lights {
			match responsive.light {
				Light::Point(cstr) => insert_light(&mut commands, entity, cstr),
				Light::Spot(cstr) => insert_light(&mut commands, entity, cstr),
				Light::Directional(cstr) => insert_light(&mut commands, entity, cstr),
			}
		}
	}
}

impl PartialEq for ResponsiveLight {
	fn eq(&self, other: &Self) -> bool {
		self.light == other.light
			&& self.range == other.range
			&& self.max == other.max
			&& self.change == other.change
			&& self.marker_on == other.marker_on
			&& self.marker_off == other.marker_off
			&& std::ptr::fn_addr_eq(self.light_on_material, other.light_on_material)
			&& std::ptr::fn_addr_eq(self.light_off_material, other.light_off_material)
	}
}

impl Prefab<()> for ResponsiveLight {
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Unreachable> {
		entity.try_insert_if_new(Collider::ball(*self.range.deref()));

		Ok(())
	}
}

struct LightOn;

struct LightOff;

#[derive(PartialEq)]
enum State {
	Done,
	Busy,
}

fn apply_change<TLight>(
	entity: &mut EntityCommands,
	change: &ResponsiveLightChange,
	responsive: &ResponsiveLight,
	light: &mut TLight,
	delta: Duration,
) -> State
where
	TLight: LightComponent,
{
	match change {
		ResponsiveLightChange::Increase => increase(entity, responsive, light, delta),
		ResponsiveLightChange::Decrease => decrease(entity, responsive, light, delta),
	}
}

fn remove_change_component(entity: &mut EntityCommands, state: State) {
	if state != State::Done {
		return;
	}
	entity.remove::<ResponsiveLightChange>();
}

fn increase<TLight>(
	entity: &mut EntityCommands,
	responsive: &ResponsiveLight,
	light: &mut TLight,
	delta: Duration,
) -> State
where
	TLight: LightComponent,
{
	let intensity = light.intensity_mut();

	if intensity == &0. {
		entity.try_insert(InsertAsset::shared_id(
			responsive.light_on_material,
			responsive.marker_on,
		));
	}

	let change = *responsive.change * delta.as_secs_f32();
	let max = *responsive.max;
	if max - *intensity > change {
		*intensity += change;
		return State::Busy;
	}

	*intensity = max;

	State::Done
}

fn decrease<TLight>(
	entity: &mut EntityCommands,
	responsive: &ResponsiveLight,
	light: &mut TLight,
	delta: Duration,
) -> State
where
	TLight: LightComponent,
{
	let intensity = light.intensity_mut();

	let change = *responsive.change * delta.as_secs_f32();
	if change < *intensity {
		*intensity -= change;
		return State::Busy;
	}

	*intensity = 0.;
	entity.try_insert(InsertAsset::shared_id(
		responsive.light_off_material,
		responsive.marker_off,
	));

	State::Done
}

fn get_change<TColliderCollection: HasCollisions>(
	colliders: &TColliderCollection,
	triggers: &Query<&ResponsiveLightTrigger>,
) -> ResponsiveLightChange {
	if colliders.collisions().any(|e| triggers.contains(e)) {
		return ResponsiveLightChange::Increase;
	}

	ResponsiveLightChange::Decrease
}

fn insert_light<TLight>(commands: &mut ZyheedaCommands, entity: Entity, cstr: fn() -> TLight)
where
	TLight: LightComponent,
{
	commands.try_apply_on(&entity, |mut e| {
		let mut light = cstr();
		*light.intensity_mut() = 0.;
		e.try_insert(light);
	});
}

#[cfg(test)]
mod test_detect_change {
	use super::*;
	use common::tools::{Intensity, IntensityChangePerSecond, Units};
	use testing::SingleThreadedApp;

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

	fn light_on_material() -> StandardMaterial {
		StandardMaterial {
			base_color: Color::WHITE,
			..default()
		}
	}

	fn light_off_material() -> StandardMaterial {
		StandardMaterial {
			base_color: Color::BLACK,
			..default()
		}
	}

	struct _MarkerOn;

	struct _MarkerOff;

	#[test]
	fn apply_on() {
		let mut app = setup();
		let trigger = app.world_mut().spawn(ResponsiveLightTrigger).id();
		let responsive = ResponsiveLight {
			light: Light::Point(PointLight::default),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};

		let entity = app
			.world_mut()
			.spawn((responsive.clone(), _Collisions(vec![trigger])))
			.id();

		app.update();

		assert_eq!(
			Some(&ResponsiveLightChange::Increase),
			app.world().entity(entity).get::<ResponsiveLightChange>(),
		)
	}

	#[test]
	fn apply_off() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: Light::Point(PointLight::default),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};

		let entity = app
			.world_mut()
			.spawn((responsive.clone(), _Collisions(vec![])))
			.id();

		app.update();

		assert_eq!(
			Some(&ResponsiveLightChange::Decrease),
			app.world().entity(entity).get::<ResponsiveLightChange>(),
		)
	}
}

#[cfg(test)]
mod test_apply_change {
	use super::*;
	use common::tools::{Intensity, IntensityChangePerSecond, Units};
	use std::time::Duration;
	use testing::{SingleThreadedApp, TickTime};

	fn light_on_material() -> StandardMaterial {
		StandardMaterial {
			base_color: Color::WHITE,
			..default()
		}
	}

	fn light_off_material() -> StandardMaterial {
		StandardMaterial {
			base_color: Color::BLACK,
			..default()
		}
	}

	struct _MarkerOn;

	struct _MarkerOff;

	#[derive(Component, Debug, PartialEq)]
	struct _Light {
		intensity: f32,
	}

	impl LightComponent for _Light {
		fn intensity_mut(&mut self) -> &mut f32 {
			&mut self.intensity
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Time<Real>>();
		app.add_systems(Update, ResponsiveLight::apply_change::<Real, _Light>);

		app
	}

	fn arbitrary_light() -> Light {
		Light::Point(PointLight::default)
	}

	#[test]
	fn increase_light_intensity() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Increase,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&_Light { intensity: 53. }),
			app.world().entity(entity).get::<_Light>()
		);
	}

	#[test]
	fn increase_light_intensity_scaled_by_delta() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Increase,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_millis(100));
		app.update();

		assert_eq!(
			Some(&_Light { intensity: 43.1 }),
			app.world().entity(entity).get::<_Light>()
		);
	}

	#[test]
	fn increase_light_intensity_clamped_at_max() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(200.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Increase,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&_Light { intensity: 100. }),
			app.world().entity(entity).get::<_Light>()
		);
	}

	#[test]
	fn set_light_on_material_when_increasing() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Increase,
				responsive,
				_Light { intensity: 0. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&InsertAsset::shared::<_MarkerOn>(light_on_material)),
			app.world()
				.entity(entity)
				.get::<InsertAsset<StandardMaterial>>()
		);
	}

	#[test]
	fn do_not_set_light_on_material_when_intensity_not_zero() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Increase,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<MeshMaterial3d<StandardMaterial>>()
				.map(|m| &m.0)
		);
	}

	#[test]
	fn remove_change_light_when_reached_max() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(58.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Increase,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<ResponsiveLightChange>()
		);
	}

	#[test]
	fn do_not_remove_change_light_when_not_reached_max() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(57.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Increase,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&ResponsiveLightChange::Increase),
			app.world().entity(entity).get::<ResponsiveLightChange>()
		);
	}

	#[test]
	fn decrease_light_intensity() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&_Light { intensity: 31. }),
			app.world().entity(entity).get::<_Light>()
		);
	}

	#[test]
	fn decrease_light_intensity_by_delta() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(11.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_millis(100));
		app.update();

		assert_eq!(
			Some(&_Light { intensity: 40.9 }),
			app.world().entity(entity).get::<_Light>()
		);
	}

	#[test]
	fn set_light_off_material_when_decreasing_to_zero() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(42.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&InsertAsset::shared::<_MarkerOff>(light_off_material)),
			app.world()
				.entity(entity)
				.get::<InsertAsset<StandardMaterial>>()
		);
	}

	#[test]
	fn do_not_set_light_off_material_when_decreasing_above_zero() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(41.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<MeshMaterial3d<StandardMaterial>>()
				.map(|m| &m.0)
		);
	}

	#[test]
	fn set_light_off_material_when_decreasing_to_below_zero() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(43.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&InsertAsset::shared::<_MarkerOff>(light_off_material)),
			app.world()
				.entity(entity)
				.get::<InsertAsset<StandardMaterial>>()
		);
	}

	#[test]
	fn decrease_light_intensity_clamped_at_zero() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(43.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&_Light { intensity: 0. }),
			app.world().entity(entity).get::<_Light>()
		);
	}

	#[test]
	fn remove_change_light_when_reached_zero() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(42.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<ResponsiveLightChange>()
		);
	}

	#[test]
	fn do_not_remove_change_light_when_not_reached_zero() {
		let mut app = setup();
		let responsive = ResponsiveLight {
			light: arbitrary_light(),
			range: Units::from(0.),
			light_on_material,
			light_off_material,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(41.),
			marker_on: TypeId::of::<_MarkerOn>(),
			marker_off: TypeId::of::<_MarkerOff>(),
		};
		let entity = app
			.world_mut()
			.spawn((
				ResponsiveLightChange::Decrease,
				responsive,
				_Light { intensity: 42. },
			))
			.id();

		app.tick_time(Duration::from_secs(1));
		app.update();

		assert_eq!(
			Some(&ResponsiveLightChange::Decrease),
			app.world().entity(entity).get::<ResponsiveLightChange>()
		);
	}
}

#[cfg(test)]
mod test_light_insertion {
	use super::*;
	use bevy::color::palettes::css::BEIGE;
	use testing::SingleThreadedApp;

	fn responsive(light: Light) -> ResponsiveLight {
		ResponsiveLight {
			light,
			range: Units::from(0.),
			light_on_material: StandardMaterial::default,
			light_off_material: StandardMaterial::default,
			max: Intensity::from(100.),
			change: IntensityChangePerSecond::from(1.),
			marker_on: TypeId::of::<()>(),
			marker_off: TypeId::of::<()>(),
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ResponsiveLight::insert_light);

		app
	}

	#[test]
	fn spawn_point_light() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(responsive(Light::Point(|| PointLight {
				color: Color::from(BEIGE),
				..default()
			})))
			.id();

		app.update();

		assert_eq!(
			Some(Color::from(BEIGE)),
			app.world()
				.entity(entity)
				.get::<PointLight>()
				.map(|l| l.color)
		);
	}

	#[test]
	fn spawn_spot_light() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(responsive(Light::Spot(|| SpotLight {
				color: Color::from(BEIGE),
				..default()
			})))
			.id();

		app.update();

		assert_eq!(
			Some(Color::from(BEIGE)),
			app.world()
				.entity(entity)
				.get::<SpotLight>()
				.map(|l| l.color)
		);
	}

	#[test]
	fn spawn_directional_light() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(responsive(Light::Directional(|| DirectionalLight {
				color: Color::from(BEIGE),
				..default()
			})))
			.id();

		app.update();

		assert_eq!(
			Some(Color::from(BEIGE)),
			app.world()
				.entity(entity)
				.get::<DirectionalLight>()
				.map(|l| l.color)
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(responsive(Light::Point(PointLight::default)))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<PointLight>();
		app.update();

		assert!(!app.world().entity(entity).contains::<PointLight>());
	}

	#[test]
	fn set_initial_intensity_zero() {
		type P = PointLight;
		type S = SpotLight;
		type D = DirectionalLight;

		let mut app = setup();
		let [a, b, c] = [
			app.world_mut()
				.spawn(responsive(Light::Point(P::default)))
				.id(),
			app.world_mut()
				.spawn(responsive(Light::Spot(S::default)))
				.id(),
			app.world_mut()
				.spawn(responsive(Light::Directional(D::default)))
				.id(),
		];

		app.update();

		assert_eq!(
			[0., 0., 0.],
			[
				app.world().entity(a).get::<P>().unwrap().intensity,
				app.world().entity(b).get::<S>().unwrap().intensity,
				app.world().entity(c).get::<D>().unwrap().illuminance,
			]
		);
	}
}
