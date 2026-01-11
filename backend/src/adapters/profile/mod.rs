//! Profile adapters for storage and analysis

mod filesystem;
mod postgres_repository;
mod postgres_reader;

pub use filesystem::FsProfileStorage;
pub use postgres_repository::PgProfileRepository;
pub use postgres_reader::PgProfileReader;
