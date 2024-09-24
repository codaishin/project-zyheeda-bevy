use bevy::prelude::*;
use common::traits::push::Push;

#[derive(Component, Default)]
pub struct EffectShaders {
	pub(crate) meshes: Vec<Handle<Mesh>>,
}

impl Push<Handle<Mesh>> for EffectShaders {
	fn push(&mut self, mesh: Handle<Mesh>) {
		self.meshes.push(mesh);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::new_handle;

	#[test]
	fn push_mesh_handle() {
		let mut shader = EffectShaders::default();
		let mesh = new_handle::<Mesh>();

		shader.push(mesh.clone());

		assert_eq!(vec![mesh], shader.meshes);
	}

	#[test]
	fn push_mesh_handles() {
		let mut shader = EffectShaders::default();
		let meshes = vec![new_handle::<Mesh>(), new_handle::<Mesh>()];

		for mesh in &meshes {
			shader.push(mesh.clone());
		}

		assert_eq!(meshes, shader.meshes);
	}
}
