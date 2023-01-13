// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use xrbk::{Readable, Writable, X11Size};

/// A message sent from an X client to the X server.
#[doc(notable_trait)]
pub trait Request: X11Size + Writable {
	/// The type representing the [other possible errors][other-errors]
	/// generated by this `Request`.
	///
	/// [other-errors]: RequestError::Other
	///
	/// All `Request`s may potentially generate, at minimum, [`Alloc`],
	/// [`Implementation`], and [`Length`] errors. As such, those errors are
	/// explicit variants of [`RequestError`].
	///
	/// If a `Request` generates no errors other than [`Alloc`],
	/// [`Implementation`], or [`Length`], this associated type should be set to
	/// [`Infallible`].
	///
	/// [`Alloc`]: crate::x11::error::Alloc
	/// [`Implementation`]: crate::x11::error::Implementation
	/// [`Length`]: crate::x11::error::Length
	///
	/// [`Infallible`]: std::convert::Infallible
	type OtherErrors;

	/// The type of [`Reply`] generated by this `Request`.
	///
	/// For `Request`s which do not generate a [reply], this is `()`.
	///
	/// [reply]: Reply
	type Reply;

	/// The major opcode that uniquely identifies this `Request` or extension.
	///
	/// X core protocol `Request`s have unique major opcodes, but each extension
	/// is only assigned one major opcode. Extensions are assigned major opcodes
	/// from 127 through to 255.
	const MAJOR_OPCODE: u8;

	/// The minor opcode that uniquely identifies this `Request` within its
	/// extension.
	///
	/// As each extension is only assigned one major opcode, the minor opcode
	/// can be used to distinguish different `Request`s contained within an
	/// extension.
	///
	/// [`Some`] means that there is indeed a minor opcode associated with this
	/// `Request`. This `Request` is therefore from an extension.
	///
	/// [`None`] means that either this request is not from an extension, or the
	/// extension does not make use of the minor opcode, likely because it only
	/// has one request.
	const MINOR_OPCODE: Option<u8>;

	/// The size of this `Request`, including the header, in 4-byte units.
	///
	/// Every `Request` contains a header which is 4 bytes long. This header is
	/// included in the `length()`, so the minimum `length()` is 1 unit (4
	/// bytes). Since the length is always in multiples of 4 bytes, padding
	/// bytes may need to be added to the end of the `Request` to ensure its
	/// `length()` is a multiple of 4 bytes.
	///
	/// The `Request` header includes the metabyte position, so that will not
	/// contribute toward the data portion.
	///
	/// |Size (excl. header)|Size (incl. header)|`length()`|
	/// |-------------------|-------------------|----------|
	/// |0                  |4                  |1         |
	/// |4                  |8                  |2         |
	/// |8                  |12                 |3         |
	/// |12                 |16                 |4         |
	/// |...                |...                |...       |
	/// |`4n - 4`           |`4n`               |`n`       |
	fn length(&self) -> u16;
}

/// The result of sending a [request].
///
/// [request]: Request
pub type RequestResult<Req: Request> = Result<Req::Reply, RequestError<Req::OtherErrors>>;

/// Represents an [error] generated by a [request].
///
/// This type exists because it is possible for _any_ [request] to generate
/// [`Alloc`], [`Implementation`], or [`Length`] errors.
///
/// [error]: Error
/// [request]: Request
pub enum RequestError<OtherErrors> {
	/// An X server may generate an [`Alloc`] event if it runs out of allocation
	/// space to allocate a requested resource.
	///
	/// An X server running out of allocation space is undefined behavior, but
	/// it is nonetheless mentioned in the X11 protocol that a server may
	/// generate an [`Alloc`] error for any [request] for this reason.
	///
	/// [`Alloc`]: crate::x11::error::Alloc
	/// [request]: Request
	Alloc(
		// error::Alloc
	),

	/// An X server may generate an [`Implementation` error] for any [request]
	/// if it does not implement some aspect of that [request].
	///
	/// The X11 protocol states that while an X server that generates an
	/// [`Implementation` error] for a [request] defined in the core X11
	/// protocol is considered "deficient", it is nonetheless a
	/// possibility.[^x11-deficient]
	///
	/// [`Implementation` error]: crate::x11::errors::Implementation
	/// [request]: Request
	///
	/// [^x11-deficient]: "A server that generates this error for a core request
	/// is deficient. As such, ...clients should be prepared to receive such
	/// errors and handle or discard them."
	/// [(X Window System Protocol, Chapter 4. Errors)][x11-errors]
	///
	/// [x11-errors]: https://x.org/releases/X11R7.7/doc/xproto/x11protocol.html#Errors:~:text=A%20server%20that%20generates%20this%20error%20for%20a%20core%20request%20is%20deficient.%20As%20such%2C%20this%20error%20is%20not%20listed%20for%20any%20of%20the%20requests%2C%20but%20clients%20should%20be%20prepared%20to%20receive%20such%20errors%20and%20handle%20or%20discard%20them.
	Implementation(
		// error::Implementation
	),

	/// An X server may generate a [`Length` error] if any [request] exceeds the
	/// maximum length accepted by the server.
	///
	/// An X server may also generate a [`Length` error] if the length of any
	/// [request] is less than the minimum or greater than the maximum length in
	/// order to encode the [request]'s data.
	///
	/// [request]: Request
	/// [`Length` error]: crate::x11::error::Length
	Length(
		// error::Length
	),

	/// Represents any other [error] generated by the X server for a [request].
	///
	/// The other possible errors for a [request] are defined by the
	/// [`Request::OtherErrors`] associated type. If no other types of [error]
	/// are generated by the [request], that type is [`Infallible`], and
	/// so, in that case, this variant cannot be created.
	///
	/// [error]: Error
	/// [request]: Request
	/// [`Infallible`]: std::convert::Infallible
	Other(OtherErrors),
}

/// A message sent from the X server to an X client in response to a
/// [`Request`].
#[doc(notable_trait)]
pub trait Reply: X11Size + Readable
where
	Self: Sized,
{
	/// The [request] that generates this `Reply`.
	///
	/// The type indicated here must implement [`Request`] with a
	/// [`Request::Reply`] associated type set to this `Reply`.
	///
	/// [request]: Request
	type Request: Request<Reply = Self>;

	/// The size of this `Reply` in 4-byte units minus 8.
	///
	/// Every `Reply` always consists of an 8-byte-long header followed by 24
	/// bytes of data, followed by zero or more additional bytes of data; this
	/// method indicates the number of additional bytes of data within the
	/// `Reply`.
	///
	/// The `Reply` header includes the metabyte position and sequence number,
	/// so those will not contribute toward the data portion.
	///
	/// |Size (excl. header)|Size (incl. header)|`length()`|
	/// |-------------------|-------------------|----------|
	/// |24                 |32                 |0         |
	/// |28                 |36                 |1         |
	/// |32                 |40                 |2         |
	/// |36                 |44                 |3         |
	/// |...                |...                |...       |
	/// |`4n - 8`           |`4n`               |`n - 8`   |
	fn length(&self) -> u32;

	/// The sequence number associated with the [request] that generated this
	/// `Reply`.
	///
	/// Every [request] on a given connection is assigned a sequence number when
	/// it is sent, starting with `1`. This sequence number can therefore be
	/// used to keep track of exactly which [request] generated this reply.
	///
	/// [request]: Request
	fn sequence(&self) -> u16;
}

/// A message sent from the X server to an X client.
///
/// `Event`s differ from [replies] in that they are not a direct response to a
/// [request] sent by the client receiving them.
///
/// [replies]: Reply
/// [request]: Request
#[doc(notable_trait)]
pub trait Event: X11Size + Readable + Writable
where
	Self: Sized,
{
	/// The code uniquely identifying this `Event`.
	const CODE: u8;

	/// The sequence number associated with the last [request] received that
	/// was related to this `Event`.
	///
	/// [request]: Request
	fn sequence(&self) -> Option<u16>;
}

/// An error sent from the X server to an X client in response to a failed
/// [request].
///
/// [request]: Request
pub trait Error: X11Size + Readable {
	/// The code uniquely identifying this `Error` (among other `Error`s).
	///
	/// Error codes 128 to 255 are reserved for extensions.
	const CODE: u8;

	/// The sequence number associated with the [request] that generated this
	/// `Error`.
	///
	/// [request]: Request
	fn sequence(&self) -> u16;

	/// The [minor opcode] of the type of [`Request`] that generated this
	/// `Error`.
	///
	/// [minor opcode]: Request::MINOR_OPCODE
	fn minor_opcode(&self) -> u16;
	/// The [major opcode] of the type of [`Request`] that generated this
	/// `Error`.
	///
	/// [major opcode]: Request::MAJOR_OPCODE
	fn major_opcode(&self) -> u8;
}
