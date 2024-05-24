use common::tools::UnitsPerSecond;

pub trait GetGravityPull {
	fn gravity_pull(&self) -> UnitsPerSecond;
}
