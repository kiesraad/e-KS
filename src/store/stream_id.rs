//! Identifier for a persisted event stream.

// Nominally `pub` in a private module (then re-exported `pub(crate)`) so the
// many nominally-`pub` signatures mentioning it don't trip `private_interfaces`.
crate::id_newtype!(pub struct StreamId);
