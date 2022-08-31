// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod errors;
mod read_value;
mod write_value;

use bytes::{Buf, BufMut, BytesMut};

pub use read_value::ReadValue;
pub use write_value::WriteValue;

pub use errors::{ReadError, ReadResult, WriteError, WriteResult};

/// Serializes a _data structure_ to bytes. _Values_ should implement [`WriteValue`] instead.
pub trait Serialize {
	/// Serializes the data structure to bytes.
	///
	/// Should have zero side effects.
	fn serialize(self) -> WriteResult<Vec<u8>>;

	/// Serializes the data structure to bytes in a [`BufMut`].
	///
	/// Do not implement this method directly: implement [`serialize()`](Serialize::serialize)
	/// instead.
	///
	/// Should have zero effects.
	fn serialize_to(self, buf: &mut impl BufMut) -> WriteResult<()>
	where
		Self: Sized,
	{
		let data = self.serialize()?;

		if buf.remaining_mut() < data.len() {
			// If there is not enough space remaining in the buffer, return a `CapacityTooLow`
			// error.
			return Err(WriteError::CapacityTooLow);
		}

		buf.put(&*data);

		Ok(())
	}
}

/// Deserializes a _data structure_ from bytes. _Values_ should implement [`ReadValue`] instead.
///
/// Implementations of [`Deserialize`] must be compatible with the existing implementation of
/// [`Serialize`] for their type.
pub trait Deserialize {
	/// Deserializes the data structure from bytes in a [`Buf`].
	///
	/// Note that bytes representing the type of data structure may not be present if they were
	/// used to determine which data structure's `deserialize` function to call.
	///
	/// Should have zero side effects.
	fn deserialize(buf: &mut impl Buf) -> ReadResult<Self>
	where
		Self: Sized;

	/// Deserializes the data structure from bytes.
	///
	/// Do not implement this function: implement
	/// [`deserialize(buf)`](Deserialize::deserialize) instead.
	fn deserialize_bytes(bytes: &[u8]) -> ReadResult<Self>
	where
		Self: Sized,
	{
		Self::deserialize(&mut bytes.clone())
	}
}

impl Serialize for String {
	fn serialize(self) -> WriteResult<Vec<u8>> {
		let mut bytes = BytesMut::new();

		// Write the string length to the byte buffer.
		self.len().write_1b_to(&mut bytes)?;

		// Write each [`char`] to the byte buffer.
		for ch in self.chars() {
			ch.write_1b_to(&mut bytes)?;
		}

		Ok(bytes.to_vec())
	}
}

impl Deserialize for String {
	fn deserialize(buf: &mut impl Buf) -> ReadResult<Self>
	where
		Self: Sized,
	{
		// Read the length of the string from the buffer.
		let length = buf.get_u8() as usize;

		// Copy `length` number of bytes to a vec.
		let bytes = buf.copy_to_bytes(length).to_vec();

		// Create a string from the bytes vec, replacing any error that occurs
		// with an [`InvalidData`] error.
		String::from_utf8(bytes).map_or(Err(ReadError::InvalidData), |text| Ok(text))
	}
}