//! Decision document module - Live markdown artifact for decision cycles.
//!
//! The Decision Document is the primary working artifact that both users and agents
//! operate on. It represents a human-readable, editable interface to the structured
//! PrOACT component data.

mod aggregate;
mod value_objects;

pub use aggregate::DecisionDocument;
pub use value_objects::{
    DocumentVersion, MarkdownContent, ParseError, ParseErrorSeverity, ParsedMetadata,
    ParsedSection, SyncSource, UpdatedBy,
};
