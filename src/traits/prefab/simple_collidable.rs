use super::complex_collidable::ComplexCollidablePrefab;

pub type SimpleCollidablePrefab<TFor, TParent, TChild> =
	ComplexCollidablePrefab<TFor, TParent, TChild, 1>;
