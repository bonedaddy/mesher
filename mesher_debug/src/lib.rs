#[macro_use]
extern crate lazy_static;

pub mod printer;
pub use printer::Printer;
pub mod inmemory;
pub use inmemory::InMemory;
