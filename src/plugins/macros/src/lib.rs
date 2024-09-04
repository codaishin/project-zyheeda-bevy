use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_macro_input, Data, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(ClampZeroPositive)]
pub fn clamp_zero_positive_derive(input: TokenStream) -> TokenStream {
	let ast: DeriveInput = parse(input).unwrap();
	let ident = ast.ident;
	let implementation = quote! {
		impl ClampZeroPositive for #ident {
			fn new(value: f32) -> Self {
				if value < 0. {
					Self(0.)
				} else {
					Self(value)
				}
			}
		}

		impl Default for #ident {
			fn default() -> Self {
				Self(0.)
			}
		}

		impl Deref for #ident {
			type Target = f32;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}
	};

	implementation.into()
}

#[proc_macro_derive(NestedMock)]
pub fn nested_mock_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let (mock_field_ident, mock_field_type) = match get_mock_field(&input) {
		Ok(value) => value,
		Err(error) => return TokenStream::from(error),
	};
	let ident = &input.ident;

	let implementation = quote! {
		impl common::traits::nested_mock::NestedMock<#mock_field_type> for #ident {
			fn new_mock(mut configure_mock_fn: impl FnMut(&mut #mock_field_type)) -> Self {
				let mut mock = #mock_field_type::default();
				configure_mock_fn(&mut mock);

				Self { #mock_field_ident: mock }
			}
		}
	};

	implementation.into()
}

#[proc_macro_derive(NestedMocks)]
pub fn nested_mocks_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let fields = match get_fields(&input) {
		Ok(fields) => fields,
		Err(error) => return error.into(),
	};

	let ident = &input.ident;
	let mut inits = Vec::new();
	let mut traits = Vec::new();

	for (field_name, ty) in fields {
		inits.push(quote! {
			#field_name: #ty::default(),
		});
		traits.push(quote! {
			impl common::traits::nested_mock::NestedMocks<#ty> for #ident {
				fn with_mock(mut self, mut configure_fn: impl FnMut(&mut #ty)) -> Self {
					configure_fn(&mut self.#field_name);
					self
				}
			}
		})
	}

	let implementation = quote! {
		impl #ident {
			pub fn new() -> Self {
				Self { #(#inits)* }
			}

		}

		#(#traits)*
	};

	implementation.into()
}

enum SetupMockCompileError {
	NotAStruct,
	NoNamedFields,
	FaultyFields,
}

impl From<SetupMockCompileError> for TokenStream {
	fn from(value: SetupMockCompileError) -> Self {
		match value {
			SetupMockCompileError::NotAStruct => TokenStream::from(quote! {
				compile_error!("SetupMock(s) can only be derived for structs");
			}),
			SetupMockCompileError::NoNamedFields => TokenStream::from(quote! {
				compile_error!("SetupMock(s) can only be derived for structs with named fields");
			}),
			SetupMockCompileError::FaultyFields => TokenStream::from(quote! {
				compile_error!("SetupMock can only be derived for structs with a single field named 'mock'");
			}),
		}
	}
}

fn get_mock_field(input: &DeriveInput) -> Result<(&Ident, &Type), SetupMockCompileError> {
	let fields = match get_fields(input) {
		Ok(fields) => fields,
		Err(error) => return Err(error),
	};

	let fields = fields.collect::<Vec<_>>();

	let [(ident, ty)] = fields[..] else {
		return Err(SetupMockCompileError::FaultyFields);
	};

	match ident {
		Some(ident) if ident == "mock" => Ok((ident, ty)),
		_ => Err(SetupMockCompileError::FaultyFields),
	}
}

fn get_fields(
	input: &DeriveInput,
) -> Result<impl Iterator<Item = (Option<&Ident>, &Type)>, SetupMockCompileError> {
	let Data::Struct(data_struct) = &input.data else {
		return Err(SetupMockCompileError::NotAStruct);
	};

	let Fields::Named(ref fields) = data_struct.fields else {
		return Err(SetupMockCompileError::NoNamedFields);
	};

	Ok(fields
		.named
		.iter()
		.map(|field| (field.ident.as_ref(), &field.ty)))
}
