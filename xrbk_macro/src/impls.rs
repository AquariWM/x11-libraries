// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{ts_ext::TsExt, *};

trait SerializeTokens {
	/// Generates the tokens to serialize a given item.
	fn serialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId);
}

trait DeserializeTokens {
	/// Generates the tokens to deserialize a given item.
	fn deserialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId);
}

impl Definitions {
	/// Expands the trait implementations for the given definition.
	pub fn impl_tokens(&self, tokens: &mut TokenStream2) {
		let Self(definitions) = self;

		// For each definition...
		for definition in definitions {
			match definition {
				// If it's an enum, append the serialization and
				// deserialization tokens for that enum.
				Definition::Enum(_enum) => {
					definition.serialize_tokens(tokens);
					definition.deserialize_tokens(tokens);
				}

				// Otherwise, if it's a struct, it's a little more complicated:
				// X11 messages are structs and they have a specific way to
				// serializing and deserializing them.
				Definition::Struct(r#struct) => {
					match &r#struct.metadata {
						StructMetadata::Request(_request) => {
							// TODO: (de)serialization for requests
							// request.serialize_tokens(tokens);
							// request.deserialize_tokens(tokens);

							// TODO: implement Request
							// request.impl_tokens(tokens);
						}

						StructMetadata::Reply(_reply) => {
							// TODO: (de)serialization for replies
							// reply.serialize_tokens(tokens);
							// reply.deserialize_tokens(tokens);

							// TODO: implement Reply
							// reply.impl_tokens(tokens);
						}

						StructMetadata::Event(_event) => {
							// TODO: (de)serialization for events
							// event.serialize_tokens(tokens);
							// event.deserialize_tokens(tokens);

							// TODO: implement Event
							// reply.impl_tokens(tokens);
						}

						// Just your basic, ordinary struct.
						StructMetadata::Struct(_struct) => {
							definition.serialize_tokens(tokens);
							definition.deserialize_tokens(tokens);
						}
					}
				}
			}
		}
	}
}

impl SerializeTokens for Field {
	// Tokens to serialize a field.
	fn serialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		let name = id.formatted();
		tokens.append_tokens(|| quote!(#name.write_to(writer)?;));
	}
}

impl DeserializeTokens for Field {
	// Tokens to deserialize a field.
	fn deserialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		let name = id.formatted();
		let r#type = &self.r#type;

		tokens.append_tokens(|| {
			// If this is a contextual field, that context must be provided.
			if let Some(context) = self.context() {
				let args = context.source().fmt_args();

				quote!(
					// let __my_field__ = <Vec<u8>>::read_with(
					//     reader,
					//     __my_field__(__my_len__),
					// )?;
					let #name = <#r#type as cornflakes::ContextualReadable>
						::read_with(
							reader,
							#name( #(#args,)* ),
						)?;
				)
			} else {
				quote!(
					// let __my_field2__ = u8::read_from(reader)?;
					let #name = <r#type as cornflakes::Readable>::read_from(reader)?;
				)
			}
		});
	}
}

impl SerializeTokens for Let {
	fn serialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		let name = id.formatted();
		let args = self.source.fmt_args();

		quote!(
			// __data_len__(&__data__).write_to(writer)?;
			#name( #( &#args, )* ).write_to(writer)?;
		)
		.to_tokens(tokens);
	}
}

impl DeserializeTokens for Let {
	fn deserialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		let name = id.formatted();
		let r#type = &self.r#type;

		tokens.append_tokens(|| {
			// let __data_len__: u32 = reader.read()?;
			quote!(let #name: #r#type = reader.read()?;)
		});
	}
}

impl SerializeTokens for Unused {
	fn serialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		match self {
			Self::Unit(_) => {
				// 0u8.write_to(writer)?;
				tokens.append_tokens(|| quote!(0u8.write_to(writer)?;));
			}

			Self::Array(array) => {
				let name = id.formatted();
				let args = array.source.fmt_args();

				tokens.append_tokens(|| {
					quote!(
						// writer.put_many(0u8, _unused_1_(&__data__));
						writer.put_many(
							0u8,
							#name( #(#args,)* )
						);
					)
				});
			}
		}
	}
}

impl DeserializeTokens for Unused {
	fn deserialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		tokens.append_tokens(|| {
			match self {
				Self::Array(array) => {
					let name = id.formatted();
					let args = array.source.fmt_args();

					quote!(
						// reader.advance(_unused_1_(&__data__) as usize);
						reader.advance(
							#name( #(#args,)* ) as usize,
						);
					)
				}

				Self::Unit(_) => {
					// reader.advance(1);
					quote!(reader.advance(1);)
				}
			}
		});
	}
}

impl SerializeTokens for Item {
	fn serialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		match self {
			Item::Field(field) => field.serialize_tokens(tokens, id),

			Item::Let(r#let) => r#let.serialize_tokens(tokens, id),

			Item::Unused(unused) => unused.serialize_tokens(tokens, id),
		}
	}
}

impl DeserializeTokens for Item {
	fn deserialize_tokens(&self, tokens: &mut TokenStream2, id: &ItemId) {
		match self {
			Item::Field(field) => field.deserialize_tokens(tokens, id),

			Item::Let(r#let) => r#let.deserialize_tokens(tokens, id),

			Item::Unused(unused) => unused.deserialize_tokens(tokens, id),
		}
	}
}

impl Definition {
	fn serialize_tokens(&self, tokens: &mut TokenStream2) {
		match self {
			Self::Enum(r#enum) => r#enum.serialize_tokens(tokens),
			Self::Struct(r#struct) => r#struct.serialize_tokens(tokens),
		}
	}

	fn deserialize_tokens(&self, tokens: &mut TokenStream2) {
		match self {
			Self::Enum(r#enum) => r#enum.deserialize_tokens(tokens),
			Self::Struct(r#struct) => r#struct.deserialize_tokens(tokens),
		}
	}
}

impl Enum {
	fn serialize_tokens(&self, tokens: &mut TokenStream2) {
		let name = &self.ident;

		let arms = TokenStream2::with_tokens(|tokens| {
			// Start the variants' discriminant tokens at `0`. We add `1` each
			// iteration, unless a variant explicitly specifies its
			// discriminant.
			let mut discrim = quote!(0);

			for variant in &self.variants {
				let name = &variant.ident;

				// If the variant explicitly specifies its discriminant, reset
				// the `discrim` tokens to that discriminant expression.
				if let Some((_, expr)) = &variant.discriminant {
					discrim = expr.to_token_stream();
				}

				// Tokens to destructure the variant's fields.
				let pat = TokenStream2::with_tokens(|tokens| {
					variant.items.pattern_to_tokens(tokens);
				});

				// Generate the tokens to serialize each of the variant's items.
				let inner = TokenStream2::with_tokens(|tokens| {
					for (id, item) in variant.items.pairs() {
						item.serialize_tokens(tokens, id);
					}
				});

				// Append the variant's match arm.
				tokens.append_tokens(|| {
					quote!(
						Self::#name #pat => {
							// Write the variant's discriminant (as a single byte).
							((#discrim) as u8).write_to(writer)?;

							#inner
						}
					)
				});

				// Add `1` to the discriminant tokens so that the next variant
				// starts with a discriminant one more than the current
				// variant's discriminant (unless that variant's discriminant
				// is specified explicitly).
				discrim.append_tokens(|| quote!(/* discrim */ + 1));
			}
		});

		tokens.append_tokens(|| {
			quote!(
				// impl Writable for MyEnum {
				//     fn write_to(
				//         &self,
				//         writer: &mut impl bytes::BufMut,
				//     ) -> Result<(), Box<dyn Error>> {
				//         match self {
				//             Self::Variant => {
				//                 (0 as u8).write_to(writer)?;
				//             }
				//         }
				//     }
				// }
				impl cornflakes::Writable for #name {
					fn write_to(
						&self,
						writer: &mut impl bytes::BufMut,
					) -> Result<(), Box<dyn std::error::Error>> {
						match self {
							#arms
						}
					}
				}
			)
		});
	}

	fn deserialize_tokens(&self, tokens: &mut TokenStream2) {
		let name = &self.ident;

		let arms = TokenStream2::with_tokens(|tokens| {
			let mut discrim = quote!(0);

			for variant in &self.variants {
				let name = &variant.ident;

				if let Some((_, expr)) = &variant.discriminant {
					discrim = expr.to_token_stream();
				}

				let inner = TokenStream2::with_tokens(|tokens| {
					for (id, item) in variant.items.pairs() {
						item.deserialize_tokens(tokens, id);
					}
				});

				let cons = TokenStream2::with_tokens(|tokens| {
					variant.items.constructor_to_tokens(tokens);
				});

				tokens.append_tokens(|| {
					quote!(
						#discrim => {
							#inner

							Self::#name #cons
						}
					)
				});

				discrim.append_tokens(|| quote!(/* discrim */ + 1));
			}
		});

		tokens.append_tokens(|| {
			quote!(
				impl cornflakes::Readable for #name {
					fn read_from(
						reader: &mut impl bytes::Buf,
					) -> Result<Self, Box<dyn std::error::Error>> {
						match reader.read::<u8>()? {
							#arms

							// TODO: replace with actual error
							_ => panic!("unrecognized enum variant discriminant"),
						}
					}
				}
			)
		});
	}
}

impl Struct {
	pub fn serialize_tokens(&self, tokens: &mut TokenStream2) {
		let name = self.metadata.name();

		let pat = TokenStream2::with_tokens(|tokens| {
			self.items.pattern_to_tokens(tokens);
		});

		let inner = TokenStream2::with_tokens(|tokens| {
			for (id, item) in self.items.pairs() {
				item.serialize_tokens(tokens, id);
			}
		});

		tokens.append_tokens(|| {
			quote!(
				impl cornflakes::Writable for #name {
					fn write_to(
						&self,
						writer: &mut impl bytes::BufMut,
					) -> Result<(), Box<dyn std::error::Error>> {
						let Self #pat = self;

						#inner
					}
				}
			)
		});
	}

	pub fn deserialize_tokens(&self, tokens: &mut TokenStream2) {
		let name = self.metadata.name();

		let inner = TokenStream2::with_tokens(|tokens| {
			for (id, item) in self.items.pairs() {
				item.deserialize_tokens(tokens, id);
			}
		});

		let cons = TokenStream2::with_tokens(|tokens| {
			self.items.constructor_to_tokens(tokens);
		});

		tokens.append_tokens(|| {
			quote!(
				impl cornflakes::Readable for #name {
					fn read_from(
						reader: &mut impl bytes::Buf,
					) -> Result<Self, Box<dyn std::error::Error>> {
						#inner

						Self #cons
					}
				}
			)
		});
	}
}