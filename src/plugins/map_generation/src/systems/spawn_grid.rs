use crate::components::{grid::Grid, nav_mesh::NavMesh};
use bevy::{mesh::MeshTrianglesError, prelude::*};
use common::{
	errors::{ErrorData, Level},
	tools::vec_not_nan::VecNotNan,
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::{Debug, Display};

impl NavMesh {
	#[allow(clippy::type_complexity)]
	pub(crate) fn spawn_grid<TGridGraph>(
		mut commands: ZyheedaCommands,
		meshes: Query<(Entity, &Mesh3d), (With<Self>, Without<Grid<TGridGraph>>)>,
		assets: Res<Assets<Mesh>>,
	) -> Result<(), Vec<NavMeshError<TGridGraph::TError>>>
	where
		TGridGraph: TryFromTriangles + ThreadSafe,
	{
		let mut mesh_errors = vec![];

		for (entity, Mesh3d(handle)) in meshes {
			let Some(mesh) = assets.get(handle) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				let mut mesh_error = None;
				let triangles = Self::get_mesh_triangles(entity, mesh, &mut mesh_error)
					.into_iter()
					.flatten();

				let graph = match TGridGraph::try_from_triangles(triangles) {
					Ok(graph) => graph,
					Err(error) => {
						mesh_errors.push(NavMeshError::MeshError { entity, error });
						TGridGraph::default()
					}
				};

				if let Some(mesh_error) = mesh_error {
					mesh_errors.push(mesh_error);
				}

				e.try_insert(Grid::from(graph));
			});
		}

		if !mesh_errors.is_empty() {
			return Err(mesh_errors);
		}

		Ok(())
	}

	fn get_mesh_triangles<'a, TError>(
		entity: Entity,
		mesh: &'a Mesh,
		mesh_error: &'a mut Option<NavMeshError<TError>>,
	) -> Option<impl Iterator<Item = [VecNotNan<3>; 3]>> {
		let triangles = match mesh.triangles() {
			Err(error) => {
				*mesh_error = Some(NavMeshError::TriangleError { entity, error });
				return None;
			}
			Ok(triangles) => triangles,
		};

		Some(triangles.filter_map(move |Triangle3d { vertices }| {
			Some([
				Self::vec_to_vec_nan(vertices[0], entity, mesh_error)?,
				Self::vec_to_vec_nan(vertices[1], entity, mesh_error)?,
				Self::vec_to_vec_nan(vertices[2], entity, mesh_error)?,
			])
		}))
	}

	fn vec_to_vec_nan<TError>(
		v: Vec3,
		entity: Entity,
		mesh_error: &mut Option<NavMeshError<TError>>,
	) -> Option<VecNotNan<3>> {
		match VecNotNan::try_from(v) {
			Ok(v) => Some(v),
			Err(_) => {
				*mesh_error = Some(NavMeshError::HasNaNVertices { entity });
				None
			}
		}
	}
}

pub(crate) trait TryFromTriangles: Sized + Default {
	type TError: Display;

	fn try_from_triangles<TIterator>(triangles: TIterator) -> Result<Self, Self::TError>
	where
		TIterator: Iterator<Item = [VecNotNan<3>; 3]>;
}

#[derive(Debug)]
pub(crate) enum NavMeshError<TMeshError> {
	TriangleError {
		entity: Entity,
		error: MeshTrianglesError,
	},
	MeshError {
		entity: Entity,
		error: TMeshError,
	},
	HasNaNVertices {
		entity: Entity,
	},
}

impl<TError> Display for NavMeshError<TError>
where
	TError: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			NavMeshError::TriangleError { entity, error } => write!(f, "{entity:?}: {error}"),
			NavMeshError::MeshError { entity, error } => write!(f, "{entity:?}: {error}"),
			NavMeshError::HasNaNVertices { entity } => {
				write!(f, "{entity:?}: has vertices that are `NaN`")
			}
		}
	}
}

impl<TError> ErrorData for NavMeshError<TError>
where
	TError: Display,
{
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Nav Mesh Error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		asset::RenderAssetUsages,
		mesh::{Indices, MeshAccessError, PrimitiveTopology},
	};
	use common::{tools::vec_not_nan::VecNotNan, vec3_not_nan};
	use testing::{IsChanged, SingleThreadedApp, new_handle};

	impl<TError> PartialEq for NavMeshError<TError>
	where
		TError: PartialEq,
	{
		fn eq(&self, other: &Self) -> bool {
			match (self, other) {
				(
					Self::TriangleError {
						entity: l_entity,
						error: l_error,
					},
					Self::TriangleError {
						entity: r_entity,
						error: r_error,
					},
				) => {
					l_entity == r_entity
						&& std::mem::discriminant(l_error) == std::mem::discriminant(r_error)
				}
				(
					Self::HasNaNVertices { entity: l_entity },
					Self::HasNaNVertices { entity: r_entity },
				) => l_entity == r_entity,
				(
					Self::MeshError {
						entity: l_entity,
						error: l_error,
					},
					Self::MeshError {
						entity: r_entity,
						error: r_error,
					},
				) => l_entity == r_entity && l_error == r_error,
				_ => false,
			}
		}
	}

	#[derive(Debug, Default)]
	struct _Graph {
		triangles: Vec<[VecNotNan<3>; 3]>,
	}

	impl _Graph {
		fn sorted_triangles(&self) -> Vec<[VecNotNan<3>; 3]> {
			let mut sorted = self.triangles.clone();
			for triangle in &mut sorted {
				triangle.sort();
			}
			sorted.sort();

			sorted
		}
	}

	impl PartialEq for _Graph {
		fn eq(&self, other: &Self) -> bool {
			self.sorted_triangles() == other.sorted_triangles()
		}
	}

	impl TryFromTriangles for _Graph {
		type TError = _Error;

		fn try_from_triangles<TIterator>(triangles: TIterator) -> Result<Self, _Error>
		where
			TIterator: Iterator<Item = [VecNotNan<3>; 3]>,
		{
			Ok(Self {
				triangles: triangles.collect(),
			})
		}
	}

	#[derive(Debug, PartialEq, Default)]
	struct _FaultyGraph;

	impl TryFromTriangles for _FaultyGraph {
		type TError = _Error;

		fn try_from_triangles<TIterator>(_: TIterator) -> Result<Self, _Error> {
			Err(_Error)
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	impl Display for _Error {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "Well, that failed hard")
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<NavMeshError<_Error>>>);

	fn setup<'a, TGraph>(meshes: impl IntoIterator<Item = (&'a Handle<Mesh>, Mesh)>) -> App
	where
		TGraph: TryFromTriangles<TError = _Error> + ThreadSafe,
	{
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, asset) in meshes {
			_ = assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.add_systems(
			Update,
			(
				NavMesh::spawn_grid::<TGraph>.pipe(|In(r), mut c: Commands| {
					c.insert_resource(_Result(r));
				}),
				IsChanged::<Grid<TGraph>>::detect,
			)
				.chain(),
		);

		app
	}

	fn unit_plane() -> Mesh {
		Mesh::from(Plane3d {
			normal: Dir3::Y,
			half_size: Vec2::ONE * 0.5,
		})
	}

	#[test]
	fn spawn_plane_grid() {
		let handle = new_handle();
		let mut app = setup::<_Graph>([(&handle, unit_plane())]);
		let entity = app.world_mut().spawn((NavMesh, Mesh3d(handle))).id();

		app.update();

		assert_eq!(
			(
				Some(&Grid::from(_Graph {
					triangles: vec![
						[
							vec3_not_nan!(0.5, 0., -0.5),
							vec3_not_nan!(-0.5, 0., 0.5),
							vec3_not_nan!(-0.5, 0., -0.5),
						],
						[
							vec3_not_nan!(0.5, 0., 0.5),
							vec3_not_nan!(0.5, 0., -0.5),
							vec3_not_nan!(-0.5, 0., 0.5),
						],
					]
				})),
				&_Result(Ok(()))
			),
			(
				app.world().entity(entity).get::<Grid<_Graph>>(),
				app.world().resource::<_Result>(),
			),
		);
	}

	#[test]
	fn skip_triangles_with_nan_values() {
		fn mesh() -> Mesh {
			let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::empty());

			#[rustfmt::skip]
			let mesh = mesh
				.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![
					[0.5, f32::NAN, 0.5],
					[0.5, 0., -0.5],
					[-0.5, 0., 0.5],
					[-0.5, 0., -0.5],
				])
				.with_inserted_indices(Indices::U32(vec![
						0, 1, 2,
						1, 2, 3
				]));

			mesh
		}

		let handle = new_handle();
		let mut app = setup::<_Graph>([(&handle, mesh())]);
		let entity = app.world_mut().spawn((NavMesh, Mesh3d(handle))).id();

		app.update();

		assert_eq!(
			(
				Some(&Grid::from(_Graph {
					triangles: vec![[
						vec3_not_nan!(0.5, 0., -0.5),
						vec3_not_nan!(-0.5, 0., 0.5),
						vec3_not_nan!(-0.5, 0., -0.5),
					]]
				})),
				&_Result(Err(vec![NavMeshError::HasNaNVertices { entity }]))
			),
			(
				app.world().entity(entity).get::<Grid<_Graph>>(),
				app.world().resource::<_Result>(),
			),
		);
	}

	#[test]
	fn empty_grid_with_mesh_errors() {
		fn mesh() -> Mesh {
			Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::empty())
		}

		let handle = new_handle();
		let mut app = setup::<_Graph>([(&handle, mesh())]);
		let entity = app.world_mut().spawn((NavMesh, Mesh3d(handle))).id();

		app.update();

		assert_eq!(
			(
				Some(&Grid::from(_Graph { triangles: vec![] })),
				&_Result(Err(vec![NavMeshError::TriangleError {
					entity,
					error: MeshTrianglesError::MeshAccessError(MeshAccessError::NotFound)
				}]))
			),
			(
				app.world().entity(entity).get::<Grid<_Graph>>(),
				app.world().resource::<_Result>(),
			),
		);
	}

	#[test]
	fn return_graph_instantiation_error() {
		let handle = new_handle();
		let mut app = setup::<_FaultyGraph>([(&handle, unit_plane())]);
		let entity = app.world_mut().spawn((NavMesh, Mesh3d(handle))).id();

		app.update();

		assert_eq!(
			(
				Some(&Grid::from(_FaultyGraph)),
				&_Result(Err(vec![NavMeshError::MeshError {
					entity,
					error: _Error
				}]))
			),
			(
				app.world().entity(entity).get::<Grid<_FaultyGraph>>(),
				app.world().resource::<_Result>(),
			),
		);
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle();
		let mut app = setup::<_Graph>([(&handle, unit_plane())]);
		let entity = app.world_mut().spawn((NavMesh, Mesh3d(handle))).id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<Grid<_Graph>>>()
		);
	}

	#[test]
	fn do_nothing_when_no_nav_mesh_component_present() {
		let handle = new_handle();
		let mut app = setup::<_Graph>([(&handle, unit_plane())]);
		let entity = app.world_mut().spawn(Mesh3d(handle)).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<Grid<_Graph>>());
	}
}
