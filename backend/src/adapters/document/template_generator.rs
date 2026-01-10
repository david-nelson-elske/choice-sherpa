//! Template-based document generator adapter.
//!
//! Generates decision documents from cycle state using a fixed template structure.
//! This is the primary implementation of the DocumentGenerator port.

use serde_json::Value;

use crate::domain::cycle::Cycle;
use crate::domain::foundation::ComponentType;
use crate::ports::{DocumentError, DocumentFormat, DocumentGenerator, GenerationOptions};

/// Template-based implementation of DocumentGenerator.
///
/// Uses a fixed markdown template structure that maps to the PrOACT framework.
/// Each component type has a corresponding section in the document.
#[derive(Debug, Clone, Default)]
pub struct TemplateDocumentGenerator {
    // Configuration could be added here for customization
}

impl TemplateDocumentGenerator {
    /// Creates a new template document generator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates the Issue Raising section.
    fn generate_issue_raising(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 1. Issue Raising\n\n");

        if let Some(data) = output {
            // Synthesis
            if let Some(synthesis) = data.get("synthesis").and_then(|v| v.as_str()) {
                section.push_str(&format!("> {}\n\n", synthesis));
            }

            // Decisions
            if let Some(decisions) = data.get("decisions").and_then(|v| v.as_array()) {
                section.push_str("### Potential Decisions\n");
                for d in decisions {
                    if let Some(text) = d.as_str() {
                        section.push_str(&format!("- [ ] {}\n", text));
                    }
                }
                section.push('\n');
            }

            // Objectives
            if let Some(objectives) = data.get("objectives").and_then(|v| v.as_array()) {
                section.push_str("### Objectives Identified\n");
                for o in objectives {
                    if let Some(text) = o.as_str() {
                        section.push_str(&format!("- {}\n", text));
                    }
                }
                section.push('\n');
            }

            // Uncertainties
            if let Some(uncertainties) = data.get("uncertainties").and_then(|v| v.as_array()) {
                section.push_str("### Uncertainties\n");
                for u in uncertainties {
                    if let Some(text) = u.as_str() {
                        section.push_str(&format!("- {}\n", text));
                    }
                }
                section.push('\n');
            }

            // Considerations
            if let Some(considerations) = data.get("considerations").and_then(|v| v.as_array()) {
                section.push_str("### Considerations\n");
                for c in considerations {
                    if let Some(text) = c.as_str() {
                        section.push_str(&format!("- {}\n", text));
                    }
                }
                section.push('\n');
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Problem Frame section.
    fn generate_problem_frame(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 2. Problem Frame\n\n");

        if let Some(data) = output {
            // Decision Maker
            if let Some(dm) = data.get("decision_maker") {
                let name = dm.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let role = dm.get("role").and_then(|v| v.as_str()).unwrap_or("Decision Maker");
                section.push_str(&format!("**Decision Maker:** {} ({})\n\n", name, role));
            }

            // Focal Decision
            if let Some(focal) = data.get("focal_decision").and_then(|v| v.as_str()) {
                section.push_str("**Focal Decision:**\n");
                section.push_str(&format!("> {}\n\n", focal));
            }

            // Scope
            if let Some(scope) = data.get("scope").and_then(|v| v.as_str()) {
                section.push_str(&format!("**Scope:** {}\n\n", scope));
            }

            // Deadline
            if let Some(deadline) = data.get("deadline").and_then(|v| v.as_str()) {
                section.push_str(&format!("**Deadline:** {}\n\n", deadline));
            }

            // Decision Hierarchy
            if let Some(hierarchy) = data.get("decision_hierarchy").and_then(|v| v.as_array()) {
                section.push_str("### Decision Hierarchy\n");
                section.push_str("| Level | Decision | Status |\n");
                section.push_str("|-------|----------|--------|\n");
                for h in hierarchy {
                    let level = h.get("level").and_then(|v| v.as_str()).unwrap_or("Unknown");
                    let decision = h.get("decision").and_then(|v| v.as_str()).unwrap_or("");
                    let status = h.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("| {} | {} | {} |\n", level, decision, status));
                }
                section.push('\n');
            }

            // Parties Involved
            if let Some(parties) = data.get("parties").and_then(|v| v.as_array()) {
                section.push_str("### Parties Involved\n");
                section.push_str("| Name | Role | Key Concerns |\n");
                section.push_str("|------|------|--------------||\n");
                for p in parties {
                    let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                    let role = p.get("role").and_then(|v| v.as_str()).unwrap_or("");
                    let concerns = p.get("concerns").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("| {} | {} | {} |\n", name, role, concerns));
                }
                section.push('\n');
            }

            // Constraints
            if let Some(constraints) = data.get("constraints").and_then(|v| v.as_array()) {
                section.push_str("### Constraints\n");
                for c in constraints {
                    let ctype = c.get("type").and_then(|v| v.as_str()).unwrap_or("General");
                    let desc = c.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("- **{}:** {}\n", ctype, desc));
                }
                section.push('\n');
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Objectives section.
    fn generate_objectives(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 3. Objectives\n\n");

        if let Some(data) = output {
            // Fundamental Objectives
            if let Some(fundamental) = data.get("fundamental").and_then(|v| v.as_array()) {
                section.push_str("### Fundamental Objectives (What Really Matters)\n\n");
                section.push_str("| Objective | Measure | Direction |\n");
                section.push_str("|-----------|---------|----------|\n");
                for obj in fundamental {
                    let name = obj.get("objective").and_then(|v| v.as_str()).unwrap_or("");
                    let measure = obj.get("measure").and_then(|v| v.as_str()).unwrap_or("");
                    let direction = obj
                        .get("direction")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    section.push_str(&format!("| {} | {} | {} |\n", name, measure, direction));
                }
                section.push('\n');
            }

            // Means Objectives
            if let Some(means) = data.get("means").and_then(|v| v.as_array()) {
                section.push_str("### Means Objectives (Ways to Achieve)\n\n");
                section.push_str("| Means Objective | Supports |\n");
                section.push_str("|-----------------|----------|\n");
                for obj in means {
                    let name = obj.get("objective").and_then(|v| v.as_str()).unwrap_or("");
                    let supports = obj.get("supports").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("| {} | {} |\n", name, supports));
                }
                section.push('\n');
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Alternatives section.
    fn generate_alternatives(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 4. Alternatives\n\n");

        if let Some(data) = output {
            // Alternatives list
            if let Some(alternatives) = data.get("alternatives").and_then(|v| v.as_array()) {
                section.push_str("### Options Under Consideration\n\n");
                section.push_str("| # | Alternative | Description | Status Quo? |\n");
                section.push_str("|---|-------------|-------------|-------------|\n");
                for (i, alt) in alternatives.iter().enumerate() {
                    let letter = (b'A' + i as u8) as char;
                    let name = alt.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let desc = alt.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    let status_quo = alt
                        .get("is_status_quo")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let sq_marker = if status_quo { "**Yes**" } else { "No" };
                    section.push_str(&format!(
                        "| {} | {} | {} | {} |\n",
                        letter, name, desc, sq_marker
                    ));
                }
                section.push('\n');
            }

            // Strategy Table
            if let Some(strategy) = data.get("strategy_table") {
                if let Some(sub_decisions) = strategy.get("sub_decisions").and_then(|v| v.as_array())
                {
                    section.push_str("### Strategy Table\n\n");
                    // Build header based on alternatives count
                    let alt_count = data
                        .get("alternatives")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    let mut header = String::from("| Sub-Decision |");
                    let mut separator = String::from("|--------------|");
                    for i in 0..alt_count {
                        let letter = (b'A' + i as u8) as char;
                        header.push_str(&format!(" {} |", letter));
                        separator.push_str("---|");
                    }
                    section.push_str(&format!("{}\n{}\n", header, separator));

                    for sub in sub_decisions {
                        let name = sub.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let mut row = format!("| {} |", name);
                        if let Some(options) = sub.get("options").and_then(|v| v.as_array()) {
                            for opt in options {
                                let value = opt.as_str().unwrap_or("");
                                row.push_str(&format!(" {} |", value));
                            }
                        }
                        section.push_str(&format!("{}\n", row));
                    }
                    section.push('\n');
                }
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Consequences section.
    fn generate_consequences(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 5. Consequences\n\n");

        if let Some(data) = output {
            // Consequence Matrix (Pugh Analysis)
            if let Some(matrix) = data.get("consequence_matrix") {
                section.push_str("### Consequence Matrix (Pugh Analysis)\n\n");

                // Get alternatives and objectives
                if let Some(alternatives) = matrix.get("alternatives").and_then(|v| v.as_array()) {
                    // Build header
                    let mut header = String::from("| Objective |");
                    let mut separator = String::from("|-----------|");
                    for alt in alternatives {
                        let name = alt.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        header.push_str(&format!(" {} |", name));
                        separator.push_str(":---:|");
                    }
                    section.push_str(&format!("{}\n{}\n", header, separator));

                    // Build rows from objectives
                    if let Some(objectives) = matrix.get("objectives").and_then(|v| v.as_array()) {
                        for obj in objectives {
                            let obj_name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let mut row = format!("| {} |", obj_name);
                            if let Some(scores) = obj.get("scores").and_then(|v| v.as_array()) {
                                for score in scores {
                                    let s = score.as_i64().unwrap_or(0);
                                    let formatted = if s > 0 {
                                        format!("+{}", s)
                                    } else if s == 0 {
                                        "0".to_string()
                                    } else {
                                        s.to_string()
                                    };
                                    row.push_str(&format!(" {} |", formatted));
                                }
                            }
                            section.push_str(&format!("{}\n", row));
                        }

                        // Add total row
                        let mut total_row = String::from("| **Total** |");
                        if let Some(totals) = matrix.get("totals").and_then(|v| v.as_array()) {
                            for total in totals {
                                let t = total.as_i64().unwrap_or(0);
                                let formatted = if t > 0 {
                                    format!("**+{}**", t)
                                } else if t == 0 {
                                    "0".to_string()
                                } else {
                                    format!("**{}**", t)
                                };
                                total_row.push_str(&format!(" {} |", formatted));
                            }
                        }
                        section.push_str(&format!("{}\n\n", total_row));
                    }
                }

                section.push_str(
                    "**Rating Scale:** -2 (Much Worse) → -1 (Worse) → 0 (Same) → +1 (Better) → +2 (Much Better)\n\n",
                );
            }

            // Key Uncertainties
            if let Some(uncertainties) = data.get("uncertainties").and_then(|v| v.as_array()) {
                section.push_str("### Key Uncertainties\n");
                section.push_str("| Uncertainty | Impact | Resolvable? |\n");
                section.push_str("|-------------|--------|-------------|\n");
                for u in uncertainties {
                    let desc = u.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    let impact = u.get("impact").and_then(|v| v.as_str()).unwrap_or("");
                    let resolvable = u
                        .get("resolvable")
                        .and_then(|v| v.as_bool())
                        .map(|b| if b { "Yes" } else { "No" })
                        .unwrap_or("Unknown");
                    section.push_str(&format!("| {} | {} | {} |\n", desc, impact, resolvable));
                }
                section.push('\n');
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Tradeoffs section.
    fn generate_tradeoffs(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 6. Tradeoffs\n\n");

        if let Some(data) = output {
            // Dominated Alternatives
            if let Some(dominated) = data.get("dominated_alternatives").and_then(|v| v.as_array()) {
                section.push_str("### Dominated Alternatives\n");
                if dominated.is_empty() {
                    section.push_str("- **None** - All alternatives have distinct advantages\n\n");
                } else {
                    for d in dominated {
                        let name = d.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let reason = d.get("reason").and_then(|v| v.as_str()).unwrap_or("");
                        section.push_str(&format!("- **{}** - {}\n", name, reason));
                    }
                    section.push('\n');
                }
            }

            // Irrelevant Objectives
            if let Some(irrelevant) = data.get("irrelevant_objectives").and_then(|v| v.as_array()) {
                section.push_str("### Irrelevant Objectives\n");
                if irrelevant.is_empty() {
                    section.push_str("- **None** - All objectives differentiate options\n\n");
                } else {
                    for i in irrelevant {
                        if let Some(name) = i.as_str() {
                            section.push_str(&format!("- **{}**\n", name));
                        }
                    }
                    section.push('\n');
                }
            }

            // Key Tensions
            if let Some(tensions) = data.get("key_tensions").and_then(|v| v.as_array()) {
                section.push_str("### Key Tensions\n");
                section.push_str("| Alternative | Excels At | Sacrifices |\n");
                section.push_str("|-------------|-----------|------------|\n");
                for t in tensions {
                    let alt = t.get("alternative").and_then(|v| v.as_str()).unwrap_or("");
                    let excels = t.get("excels_at").and_then(|v| v.as_str()).unwrap_or("");
                    let sacrifices = t.get("sacrifices").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("| {} | {} | {} |\n", alt, excels, sacrifices));
                }
                section.push('\n');
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Recommendation section.
    fn generate_recommendation(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 7. Recommendation\n\n");

        if let Some(data) = output {
            // Synthesis
            if let Some(synthesis) = data.get("synthesis").and_then(|v| v.as_str()) {
                section.push_str("### Synthesis\n");
                section.push_str(&format!("> {}\n\n", synthesis));
            }

            // Standout Option
            if let Some(standout) = data.get("standout_option") {
                let name = standout.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let rationale = standout
                    .get("rationale")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                section.push_str("### If One Stands Out\n");
                section.push_str(&format!("- **Standout Option:** {}\n", name));
                section.push_str(&format!("- **Rationale:** {}\n\n", rationale));
            }

            // Key Considerations
            if let Some(considerations) = data.get("key_considerations").and_then(|v| v.as_array())
            {
                section.push_str("### Key Considerations Before Deciding\n");
                for (i, c) in considerations.iter().enumerate() {
                    if let Some(text) = c.as_str() {
                        section.push_str(&format!("{}. {}\n", i + 1, text));
                    }
                }
                section.push('\n');
            }

            // Remaining Uncertainties
            if let Some(uncertainties) = data.get("remaining_uncertainties").and_then(|v| v.as_array())
            {
                section.push_str("### Remaining Uncertainties\n");
                for u in uncertainties {
                    let desc = u.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    let resolution = u.get("resolution_path").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("- {} - {}\n", desc, resolution));
                }
                section.push('\n');
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Decision Quality section.
    fn generate_decision_quality(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## 8. Decision Quality Assessment\n\n");

        if let Some(data) = output {
            // Element scores
            if let Some(elements) = data.get("elements").and_then(|v| v.as_array()) {
                section.push_str("| Element | Score | Rationale |\n");
                section.push_str("|---------|:-----:|----------|\n");

                let mut min_score: Option<u8> = None;
                let mut min_element: Option<&str> = None;

                for e in elements {
                    let name = e.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let score = e.get("score").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                    let rationale = e.get("rationale").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("| {} | {}% | {} |\n", name, score, rationale));

                    if min_score.map_or(true, |m| score < m) {
                        min_score = Some(score);
                        min_element = Some(name);
                    }
                }

                // Overall Quality (weakest link)
                let overall = min_score.unwrap_or(0);
                let weakest = min_element.unwrap_or("Unknown");
                section.push_str(&format!(
                    "| **Overall Quality** | **{}%** | *Weakest link: {}* |\n\n",
                    overall, weakest
                ));
            }

            // Improvement suggestions
            if let Some(improvements) = data.get("improvements").and_then(|v| v.as_array()) {
                section.push_str("### To Improve Quality\n");
                for imp in improvements {
                    let action = imp.get("action").and_then(|v| v.as_str()).unwrap_or("");
                    let improvement = imp.get("improvement").and_then(|v| v.as_str()).unwrap_or("");
                    section.push_str(&format!("- [ ] {} ({})\n", action, improvement));
                }
                section.push('\n');
            }
        } else {
            section.push_str("*Not yet started*\n\n");
        }

        section
    }

    /// Generates the Notes & Next Steps section.
    fn generate_notes_next_steps(&self, output: Option<&Value>) -> String {
        let mut section = String::from("## Notes & Next Steps\n\n");

        if let Some(data) = output {
            // Open questions
            if let Some(questions) = data.get("open_questions").and_then(|v| v.as_array()) {
                section.push_str("### Open Questions\n");
                if questions.is_empty() {
                    section.push_str("*None recorded*\n\n");
                } else {
                    for q in questions {
                        if let Some(text) = q.as_str() {
                            section.push_str(&format!("- {}\n", text));
                        }
                    }
                    section.push('\n');
                }
            }

            // Next steps
            if let Some(steps) = data.get("next_steps").and_then(|v| v.as_array()) {
                section.push_str("### Next Steps\n");
                if steps.is_empty() {
                    section.push_str("*None recorded*\n\n");
                } else {
                    for step in steps {
                        if let Some(text) = step.as_str() {
                            section.push_str(&format!("- [ ] {}\n", text));
                        }
                    }
                    section.push('\n');
                }
            }

            // Revisit conditions
            if let Some(conditions) = data.get("revisit_conditions").and_then(|v| v.as_array()) {
                section.push_str("### When to Revisit This Decision\n");
                if conditions.is_empty() {
                    section.push_str("*Conditions not yet defined*\n\n");
                } else {
                    for c in conditions {
                        if let Some(text) = c.as_str() {
                            section.push_str(&format!("- {}\n", text));
                        }
                    }
                    section.push('\n');
                }
            }
        } else {
            section.push_str("### Open Questions\n");
            section.push_str("*None recorded*\n\n");
            section.push_str("### When to Revisit This Decision\n");
            section.push_str("*Conditions not yet defined*\n\n");
        }

        section
    }

    /// Generates content for a specific component type.
    fn generate_section_for_type(
        &self,
        component_type: ComponentType,
        output: Option<&Value>,
    ) -> String {
        match component_type {
            ComponentType::IssueRaising => self.generate_issue_raising(output),
            ComponentType::ProblemFrame => self.generate_problem_frame(output),
            ComponentType::Objectives => self.generate_objectives(output),
            ComponentType::Alternatives => self.generate_alternatives(output),
            ComponentType::Consequences => self.generate_consequences(output),
            ComponentType::Tradeoffs => self.generate_tradeoffs(output),
            ComponentType::Recommendation => self.generate_recommendation(output),
            ComponentType::DecisionQuality => self.generate_decision_quality(output),
            ComponentType::NotesNextSteps => self.generate_notes_next_steps(output),
        }
    }
}

impl DocumentGenerator for TemplateDocumentGenerator {
    fn generate(
        &self,
        session_title: &str,
        cycle: &Cycle,
        options: GenerationOptions,
    ) -> Result<String, DocumentError> {
        let mut doc = String::new();

        // Generate header
        doc.push_str(&self.generate_header(session_title, &options)?);
        doc.push_str("\n---\n\n");

        // The 8 main PrOACT components (excluding NotesNextSteps which goes in footer)
        let component_types = [
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
        ];

        for component_type in component_types {
            let output = cycle
                .component(component_type)
                .map(|c| c.output_as_value());

            // Skip empty sections in summary/export formats
            if !options.include_empty_sections && output.is_none() {
                continue;
            }

            let section = self.generate_section_for_type(component_type, output.as_ref());
            doc.push_str(&section);
            doc.push_str("---\n\n");
        }

        // Generate footer
        doc.push_str(&self.generate_footer(cycle, &options)?);

        Ok(doc)
    }

    fn generate_section(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<String, DocumentError> {
        Ok(self.generate_section_for_type(component_type, Some(output)))
    }

    fn generate_header(
        &self,
        session_title: &str,
        options: &GenerationOptions,
    ) -> Result<String, DocumentError> {
        let mut header = String::new();

        // Title
        header.push_str(&format!("# {}\n\n", session_title));

        // Status block (always included in full format)
        if options.format == DocumentFormat::Full {
            header.push_str("> **Status:** In Progress | **Quality Score:** --\n");
            header.push_str("> **Last Updated:** -- by system\n");

            if options.include_metadata {
                header.push_str("> **Cycle:** -- | **Branch:** --\n");
            }
        }

        Ok(header)
    }

    fn generate_footer(
        &self,
        _cycle: &Cycle,
        options: &GenerationOptions,
    ) -> Result<String, DocumentError> {
        let mut footer = String::new();

        if options.include_version_info {
            footer.push_str("## Notes & Next Steps\n\n");
            footer.push_str("### Open Questions\n");
            footer.push_str("*None recorded*\n\n");
            footer.push_str("### When to Revisit This Decision\n");
            footer.push_str("*Conditions not yet defined*\n\n");
            footer.push_str("---\n\n");
            footer.push_str("*Document Version: 1*\n");
            footer.push_str("*Generated by Choice Sherpa*\n");
        }

        Ok(footer)
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_generator() -> TemplateDocumentGenerator {
        TemplateDocumentGenerator::new()
    }

    // ───────────────────────────────────────────────────────────────
    // Issue Raising Section Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn issue_raising_with_no_output() {
        let gen = test_generator();
        let section = gen.generate_issue_raising(None);
        assert!(section.contains("Issue Raising"));
        assert!(section.contains("Not yet started"));
    }

    #[test]
    fn issue_raising_with_data() {
        let gen = test_generator();
        let output = json!({
            "synthesis": "A career transition decision",
            "decisions": ["Accept new job", "Stay current"],
            "objectives": ["Better compensation", "Work-life balance"],
            "uncertainties": ["Market conditions"],
            "considerations": ["Family impact"]
        });

        let section = gen.generate_issue_raising(Some(&output));
        assert!(section.contains("A career transition decision"));
        assert!(section.contains("Accept new job"));
        assert!(section.contains("Better compensation"));
        assert!(section.contains("Market conditions"));
        assert!(section.contains("Family impact"));
    }

    // ───────────────────────────────────────────────────────────────
    // Problem Frame Section Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn problem_frame_with_data() {
        let gen = test_generator();
        let output = json!({
            "decision_maker": {"name": "Alice", "role": "Manager"},
            "focal_decision": "Should I accept the VP offer?",
            "scope": "Career decision only",
            "deadline": "End of month"
        });

        let section = gen.generate_problem_frame(Some(&output));
        assert!(section.contains("Alice"));
        assert!(section.contains("VP offer"));
        assert!(section.contains("End of month"));
    }

    // ───────────────────────────────────────────────────────────────
    // Objectives Section Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn objectives_with_data() {
        let gen = test_generator();
        let output = json!({
            "fundamental": [
                {"objective": "Maximize income", "measure": "$/year", "direction": "↑ Higher is better"}
            ],
            "means": [
                {"objective": "Reduce commute", "supports": "Work-life balance"}
            ]
        });

        let section = gen.generate_objectives(Some(&output));
        assert!(section.contains("Maximize income"));
        assert!(section.contains("$/year"));
        assert!(section.contains("Reduce commute"));
    }

    // ───────────────────────────────────────────────────────────────
    // Alternatives Section Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn alternatives_with_data() {
        let gen = test_generator();
        let output = json!({
            "alternatives": [
                {"name": "Accept VP", "description": "Take the new role", "is_status_quo": false},
                {"name": "Stay current", "description": "Keep current position", "is_status_quo": true}
            ]
        });

        let section = gen.generate_alternatives(Some(&output));
        assert!(section.contains("Accept VP"));
        assert!(section.contains("Stay current"));
        assert!(section.contains("**Yes**")); // Status quo marker
    }

    // ───────────────────────────────────────────────────────────────
    // Consequences Section Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn consequences_with_pugh_matrix() {
        let gen = test_generator();
        let output = json!({
            "consequence_matrix": {
                "alternatives": [
                    {"name": "A: Accept"},
                    {"name": "B: Stay"}
                ],
                "objectives": [
                    {"name": "Income", "scores": [2, 0]},
                    {"name": "Balance", "scores": [-1, 0]}
                ],
                "totals": [1, 0]
            }
        });

        let section = gen.generate_consequences(Some(&output));
        assert!(section.contains("Pugh Analysis"));
        assert!(section.contains("+2"));
        assert!(section.contains("-1"));
        assert!(section.contains("**+1**")); // Total
    }

    // ───────────────────────────────────────────────────────────────
    // Decision Quality Section Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn decision_quality_with_scores() {
        let gen = test_generator();
        let output = json!({
            "elements": [
                {"name": "Clear Problem Frame", "score": 85, "rationale": "Well defined"},
                {"name": "Clear Objectives", "score": 70, "rationale": "Needs measures"}
            ]
        });

        let section = gen.generate_decision_quality(Some(&output));
        assert!(section.contains("85%"));
        assert!(section.contains("70%"));
        assert!(section.contains("**70%**")); // Overall (min)
    }

    // ───────────────────────────────────────────────────────────────
    // Header and Footer Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn header_includes_title() {
        let gen = test_generator();
        let options = GenerationOptions::full();
        let header = gen.generate_header("Career Decision", &options).unwrap();
        assert!(header.contains("# Career Decision"));
        assert!(header.contains("Status:"));
    }

    #[test]
    fn footer_includes_version() {
        let gen = test_generator();
        let options = GenerationOptions::full();

        use crate::domain::foundation::SessionId;
        let cycle = Cycle::new(SessionId::new());

        let footer = gen.generate_footer(&cycle, &options).unwrap();
        assert!(footer.contains("Document Version"));
        assert!(footer.contains("Choice Sherpa"));
    }

    // ───────────────────────────────────────────────────────────────
    // Section Generation Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn generate_section_routes_correctly() {
        let gen = test_generator();
        let output = json!({"synthesis": "Test synthesis"});

        let section = gen
            .generate_section(ComponentType::IssueRaising, &output)
            .unwrap();
        assert!(section.contains("Issue Raising"));
        assert!(section.contains("Test synthesis"));
    }
}
