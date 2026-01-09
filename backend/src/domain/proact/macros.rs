//! DRY macros for PrOACT components.
//!
//! These macros eliminate boilerplate in component implementations:
//!
//! - **`impl_component!`** - Generates the `Component` trait implementation (~40 lines saved per component)
//! - **`delegate_to_variant!`** - Generates 9-arm match blocks for `ComponentVariant` delegation
//!
//! # Usage
//!
//! ```ignore
//! // Instead of 40 lines of Component impl boilerplate:
//! impl_component!(IssueRaising, IssueRaisingOutput, ComponentType::IssueRaising);
//!
//! // Instead of repeating 9-arm matches:
//! delegate_to_variant!(self, id)
//! delegate_to_variant!(self, start)
//! delegate_to_variant!(self, mark_for_revision, reason)
//! ```

/// Implements the `Component` trait for a struct with `base: ComponentBase` and `output: T` fields.
///
/// This macro generates ~40 lines of boilerplate for each component, including:
/// - Accessor methods delegating to `base` (id, component_type, status, created_at, updated_at)
/// - Action methods delegating to `base` (start, complete, mark_for_revision)
/// - JSON serialization methods using the `output` field
///
/// # Arguments
///
/// - `$name` - The component struct name (e.g., `IssueRaising`)
/// - `$output_type` - The output struct type (e.g., `IssueRaisingOutput`)
/// - `$component_type` - The `ComponentType` variant (e.g., `ComponentType::IssueRaising`)
///
/// # Requirements
///
/// The struct must have:
/// - A `base: ComponentBase` field
/// - An `output: $output_type` field where `$output_type` implements `Serialize + DeserializeOwned + Default`
///
/// # Example
///
/// ```ignore
/// use crate::domain::proact::macros::impl_component;
///
/// pub struct MyComponent {
///     base: ComponentBase,
///     output: MyComponentOutput,
/// }
///
/// impl_component!(MyComponent, MyComponentOutput, ComponentType::MyComponent);
/// ```
#[macro_export]
macro_rules! impl_component {
    ($name:ident, $output_type:ty, $component_type:expr) => {
        impl $name {
            /// Creates a new component instance.
            pub fn new() -> Self {
                Self {
                    base: ComponentBase::new($component_type),
                    output: <$output_type>::default(),
                }
            }

            /// Returns a reference to the output.
            pub fn output(&self) -> &$output_type {
                &self.output
            }

            /// Sets the output and updates the timestamp.
            pub fn set_output(&mut self, output: $output_type) {
                self.output = output;
                self.base.touch();
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Component for $name {
            fn id(&self) -> ComponentId {
                self.base.id
            }

            fn component_type(&self) -> ComponentType {
                self.base.component_type
            }

            fn status(&self) -> ComponentStatus {
                self.base.status
            }

            fn created_at(&self) -> Timestamp {
                self.base.created_at
            }

            fn updated_at(&self) -> Timestamp {
                self.base.updated_at
            }

            fn start(&mut self) -> Result<(), ComponentError> {
                self.base.start()
            }

            fn complete(&mut self) -> Result<(), ComponentError> {
                self.base.complete()
            }

            fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
                self.base.mark_for_revision(reason)
            }

            fn output_as_value(&self) -> serde_json::Value {
                serde_json::to_value(&self.output).unwrap_or_default()
            }

            fn set_output_from_value(&mut self, value: serde_json::Value) -> Result<(), ComponentError> {
                self.output = serde_json::from_value(value)
                    .map_err(|e| ComponentError::InvalidOutput(e.to_string()))?;
                self.base.touch();
                Ok(())
            }
        }
    };
}

/// Delegates a method call to all variants in `ComponentVariant`.
///
/// This macro generates a 9-arm match block that calls the same method on each
/// variant's inner component. Eliminates the repetitive pattern of:
///
/// ```ignore
/// match self {
///     ComponentVariant::IssueRaising(c) => c.method(),
///     ComponentVariant::ProblemFrame(c) => c.method(),
///     // ... 7 more arms
/// }
/// ```
///
/// # Variants
///
/// - `delegate_to_variant!(self, method)` - For methods with no arguments
/// - `delegate_to_variant!(self, method, arg1, arg2, ...)` - For methods with arguments
///
/// # Example
///
/// ```ignore
/// impl ComponentVariant {
///     pub fn id(&self) -> ComponentId {
///         delegate_to_variant!(self, id)
///     }
///
///     pub fn start(&mut self) -> Result<(), ComponentError> {
///         delegate_to_variant!(self, start)
///     }
///
///     pub fn mark_for_revision(&mut self, reason: String) -> Result<(), ComponentError> {
///         delegate_to_variant!(self, mark_for_revision, reason)
///     }
/// }
/// ```
#[macro_export]
macro_rules! delegate_to_variant {
    // Version without arguments
    ($self:expr, $method:ident) => {
        match $self {
            ComponentVariant::IssueRaising(c) => c.$method(),
            ComponentVariant::ProblemFrame(c) => c.$method(),
            ComponentVariant::Objectives(c) => c.$method(),
            ComponentVariant::Alternatives(c) => c.$method(),
            ComponentVariant::Consequences(c) => c.$method(),
            ComponentVariant::Tradeoffs(c) => c.$method(),
            ComponentVariant::Recommendation(c) => c.$method(),
            ComponentVariant::DecisionQuality(c) => c.$method(),
            ComponentVariant::NotesNextSteps(c) => c.$method(),
        }
    };

    // Version with arguments
    ($self:expr, $method:ident, $($arg:expr),+) => {
        match $self {
            ComponentVariant::IssueRaising(c) => c.$method($($arg),+),
            ComponentVariant::ProblemFrame(c) => c.$method($($arg),+),
            ComponentVariant::Objectives(c) => c.$method($($arg),+),
            ComponentVariant::Alternatives(c) => c.$method($($arg),+),
            ComponentVariant::Consequences(c) => c.$method($($arg),+),
            ComponentVariant::Tradeoffs(c) => c.$method($($arg),+),
            ComponentVariant::Recommendation(c) => c.$method($($arg),+),
            ComponentVariant::DecisionQuality(c) => c.$method($($arg),+),
            ComponentVariant::NotesNextSteps(c) => c.$method($($arg),+),
        }
    };
}

// Re-export macros for use in other modules
pub use delegate_to_variant;
pub use impl_component;

#[cfg(test)]
mod tests {
    // Tests are in the component modules that use these macros.
    // The macros are tested indirectly through their usage.
}
