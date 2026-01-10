//! Cycle module - Decision cycle aggregate and lifecycle management.
//!
//! A Cycle represents a complete or partial path through the PrOACT framework.
//! Cycles own their components and support branching for "what-if" exploration.

mod aggregate;
pub mod document;
mod events;
mod progress;

pub use aggregate::Cycle;
pub use document::{
    DecisionDocument, DocumentEvent, DocumentVersion, MarkdownContent, ParseError,
    ParseErrorSeverity, ParsedMetadata, ParsedSection, SyncSource, UpdatedBy,
};
pub use events::CycleEvent;
pub use progress::CycleProgress;
