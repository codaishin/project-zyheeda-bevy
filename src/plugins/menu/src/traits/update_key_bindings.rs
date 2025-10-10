use common::traits::handles_input::GetAllInputs;

pub(crate) trait UpdateKeyBindings {
	fn update_key_bindings<TInput>(&mut self, input: &TInput)
	where
		TInput: GetAllInputs;
}
