//! Transports which may be useful while debugging or testing mesher-based applications.
//!
//! Note that **none** of these are suitable for production use!
//! They're designed to debug and test.
//! You may be able to use them successfully outside of that context, but only at your own risk.

mod inmemory;
pub use inmemory::InMemory;
