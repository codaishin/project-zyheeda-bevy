pub trait OrOk: Sized {
	fn or_ok<TOk>(self, ok: fn() -> TOk) -> Result<TOk, Self>;
}

impl<T> OrOk for Vec<T> {
	fn or_ok<TOk>(self, ok: fn() -> TOk) -> Result<TOk, Self> {
		if !self.is_empty() {
			return Err(self);
		}

		Ok(ok())
	}
}
