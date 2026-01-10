//! Regex-based markdown document parser adapter.
//!
//! Parses decision documents back into structured PrOACT component data.
//! This is the inverse of `TemplateDocumentGenerator`.

use regex::Regex;
use serde_json::{json, Value};

use crate::domain::cycle::{ParseError, ParsedMetadata, ParsedSection};
use crate::domain::foundation::ComponentType;
use crate::ports::{DocumentError, DocumentParser, ParseResult, SectionBoundary};

/// Regex-based implementation of DocumentParser.
///
/// Uses regular expressions to parse the markdown document structure.
/// Designed to be the inverse of `TemplateDocumentGenerator` for round-trip consistency.
#[derive(Debug, Clone)]
pub struct MarkdownDocumentParser {
    section_header_regex: Regex,
    subsection_header_regex: Regex,
    table_row_regex: Regex,
    blockquote_regex: Regex,
    checkbox_regex: Regex,
    list_item_regex: Regex,
    key_value_regex: Regex,
}

impl Default for MarkdownDocumentParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownDocumentParser {
    /// Creates a new markdown document parser with precompiled regexes.
    pub fn new() -> Self {
        Self {
            // Matches "## 1. Issue Raising", "## 2. Problem Frame", etc.
            section_header_regex: Regex::new(r"^##\s+(\d+)\.\s+(.+)$").unwrap(),
            // Matches "### Subsection Title"
            subsection_header_regex: Regex::new(r"^###\s+(.+)$").unwrap(),
            // Matches table rows: "| col1 | col2 | col3 |"
            table_row_regex: Regex::new(r"^\|(.+)\|$").unwrap(),
            // Matches blockquotes: "> quoted text"
            blockquote_regex: Regex::new(r"^>\s*(.*)$").unwrap(),
            // Matches checkbox items: "- [ ] item" or "- [x] item"
            checkbox_regex: Regex::new(r"^-\s+\[([ xX])\]\s+(.+)$").unwrap(),
            // Matches list items: "- item" or "1. item"
            list_item_regex: Regex::new(r"^[-*]\s+(.+)$|^(\d+)\.\s+(.+)$").unwrap(),
            // Matches key-value: "**Key:** Value" or "**Key**: Value" or "**Key** Value"
            key_value_regex: Regex::new(r"^\*\*([^*:]+):?\*\*:?\s*(.*)$").unwrap(),
        }
    }

    /// Maps section number to ComponentType.
    fn section_number_to_type(number: u32) -> Option<ComponentType> {
        match number {
            1 => Some(ComponentType::IssueRaising),
            2 => Some(ComponentType::ProblemFrame),
            3 => Some(ComponentType::Objectives),
            4 => Some(ComponentType::Alternatives),
            5 => Some(ComponentType::Consequences),
            6 => Some(ComponentType::Tradeoffs),
            7 => Some(ComponentType::Recommendation),
            8 => Some(ComponentType::DecisionQuality),
            _ => None,
        }
    }

    /// Extracts the document title from the first H1 heading.
    fn extract_title(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
                return Some(trimmed[2..].trim().to_string());
            }
        }
        None
    }

    /// Extracts metadata from header block.
    fn extract_metadata(&self, content: &str) -> ParsedMetadata {
        let title = self.extract_title(content);
        let mut status = None;
        let mut dq_score = None;

        // Look for status line: "> **Status:** In Progress | **Quality Score:** --"
        for line in content.lines() {
            if let Some(captures) = self.blockquote_regex.captures(line) {
                let inner = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                if inner.contains("Status:") {
                    // Extract status
                    if inner.contains("In Progress") {
                        status = Some("In Progress".to_string());
                    } else if inner.contains("Complete") {
                        status = Some("Complete".to_string());
                    }
                    // Extract quality score
                    if let Some(score_start) = inner.find("Quality Score:**") {
                        let after = &inner[score_start + 16..];
                        let score_str: String = after
                            .trim()
                            .chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect();
                        if !score_str.is_empty() {
                            dq_score = score_str.parse().ok();
                        }
                    }
                }
            }
        }

        ParsedMetadata {
            title,
            focal_decision: None, // Will be extracted from ProblemFrame section
            status,
            dq_score,
        }
    }

    /// Parses the Issue Raising section.
    fn parse_issue_raising(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut synthesis = None;
        let mut decisions = Vec::new();
        let mut objectives = Vec::new();
        let mut uncertainties = Vec::new();
        let mut considerations = Vec::new();
        let mut errors = Vec::new();

        let mut current_subsection: Option<&str> = None;

        for (line_offset, line) in content.lines().enumerate() {
            let line_num = start_line + line_offset;
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                current_subsection = Some(header);
                continue;
            }

            // Handle blockquote (synthesis)
            if let Some(caps) = self.blockquote_regex.captures(trimmed) {
                let text = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                if !text.is_empty() && synthesis.is_none() {
                    synthesis = Some(text.to_string());
                }
                continue;
            }

            // Handle checkbox items (decisions)
            if let Some(caps) = self.checkbox_regex.captures(trimmed) {
                let item = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
                if !item.is_empty() {
                    if current_subsection
                        .map(|s| s.contains("Decision"))
                        .unwrap_or(false)
                    {
                        decisions.push(item.to_string());
                    }
                }
                continue;
            }

            // Handle list items
            if let Some(caps) = self.list_item_regex.captures(trimmed) {
                let item = caps
                    .get(1)
                    .or_else(|| caps.get(3))
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .trim();

                if !item.is_empty() {
                    match current_subsection {
                        Some(s) if s.contains("Objective") => objectives.push(item.to_string()),
                        Some(s) if s.contains("Uncertaint") => uncertainties.push(item.to_string()),
                        Some(s) if s.contains("Consideration") => {
                            considerations.push(item.to_string())
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check if we got any meaningful data
        let has_data = synthesis.is_some()
            || !decisions.is_empty()
            || !objectives.is_empty()
            || !uncertainties.is_empty()
            || !considerations.is_empty();

        if !has_data {
            if content.contains("Not yet started") {
                // Empty section is valid
                ParsedSection::success(ComponentType::IssueRaising, content.to_string(), json!({}))
            } else {
                errors.push(ParseError::warning(
                    start_line,
                    "Could not extract Issue Raising data",
                ));
                ParsedSection::with_errors(
                    ComponentType::IssueRaising,
                    content.to_string(),
                    errors,
                )
            }
        } else {
            // Build domain-compatible JSON (matches IssueRaisingOutput structure)
            let data = json!({
                "potential_decisions": decisions,
                "objectives": objectives,
                "uncertainties": uncertainties,
                "considerations": considerations,
                "user_confirmed": false
            });
            ParsedSection::success(ComponentType::IssueRaising, content.to_string(), data)
        }
    }

    /// Parses the Problem Frame section.
    fn parse_problem_frame(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut data = json!({});
        let mut errors = Vec::new();

        let mut in_hierarchy_table = false;
        let mut in_parties_table = false;
        let mut current_subsection: Option<&str> = None;
        let mut constraints = Vec::new();

        for (line_offset, line) in content.lines().enumerate() {
            let _line_num = start_line + line_offset;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                current_subsection = Some(header);
                in_hierarchy_table = header.contains("Decision Hierarchy");
                in_parties_table = header.contains("Parties");
                continue;
            }

            // Handle key-value pairs
            if let Some(caps) = self.key_value_regex.captures(trimmed) {
                let key = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                let value = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();

                match key {
                    "Decision Maker" => {
                        // Parse "Name (Role)"
                        if let Some(paren_start) = value.find('(') {
                            let name = value[..paren_start].trim();
                            let role = value[paren_start + 1..]
                                .trim_end_matches(')')
                                .trim();
                            data["decision_maker"] = json!({
                                "name": name,
                                "role": role
                            });
                        } else {
                            data["decision_maker"] = json!({
                                "name": value,
                                "role": "Decision Maker"
                            });
                        }
                    }
                    "Focal Decision" => {} // Handled in blockquote below
                    "Scope" => {
                        data["scope"] = json!(value);
                    }
                    "Deadline" => {
                        data["deadline"] = json!(value);
                    }
                    _ if current_subsection
                        .map(|s| s.contains("Constraint"))
                        .unwrap_or(false) =>
                    {
                        constraints.push(json!({
                            "type": key,
                            "description": value
                        }));
                    }
                    _ => {}
                }
                continue;
            }

            // Handle blockquote (focal decision)
            if let Some(caps) = self.blockquote_regex.captures(trimmed) {
                let text = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                if !text.is_empty() && !data.get("focal_decision").is_some() {
                    data["focal_decision"] = json!(text);
                }
                continue;
            }

            // Handle table rows
            if in_hierarchy_table || in_parties_table {
                if let Some(caps) = self.table_row_regex.captures(trimmed) {
                    let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    // Skip separator rows
                    if inner.contains("---") {
                        continue;
                    }
                    let cells: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();

                    if in_hierarchy_table && cells.len() >= 3 {
                        // Skip header row
                        if cells[0] != "Level" {
                            let hierarchy = data
                                .get_mut("decision_hierarchy")
                                .and_then(|v| v.as_array_mut());
                            let entry = json!({
                                "level": cells[0],
                                "decision": cells[1],
                                "status": cells[2]
                            });
                            if let Some(arr) = hierarchy {
                                arr.push(entry);
                            } else {
                                data["decision_hierarchy"] = json!([entry]);
                            }
                        }
                    }

                    if in_parties_table && cells.len() >= 3 {
                        // Skip header row
                        if cells[0] != "Name" {
                            let parties = data.get_mut("parties").and_then(|v| v.as_array_mut());
                            let entry = json!({
                                "name": cells[0],
                                "role": cells[1],
                                "concerns": cells[2]
                            });
                            if let Some(arr) = parties {
                                arr.push(entry);
                            } else {
                                data["parties"] = json!([entry]);
                            }
                        }
                    }
                }
            }
        }

        if !constraints.is_empty() {
            data["constraints"] = json!(constraints);
        }

        if content.contains("Not yet started") {
            ParsedSection::success(ComponentType::ProblemFrame, content.to_string(), json!({}))
        } else if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            errors.push(ParseError::warning(
                start_line,
                "Could not extract Problem Frame data",
            ));
            ParsedSection::with_errors(ComponentType::ProblemFrame, content.to_string(), errors)
        } else {
            ParsedSection::success(ComponentType::ProblemFrame, content.to_string(), data)
        }
    }

    /// Parses the Objectives section.
    fn parse_objectives(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut fundamental = Vec::new();
        let mut means = Vec::new();
        let mut errors = Vec::new();

        let mut in_fundamental = false;
        let mut in_means = false;

        for (line_offset, line) in content.lines().enumerate() {
            let _line_num = start_line + line_offset;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                in_fundamental = header.contains("Fundamental");
                in_means = header.contains("Means");
                continue;
            }

            // Handle table rows
            if let Some(caps) = self.table_row_regex.captures(trimmed) {
                let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                if inner.contains("---") {
                    continue;
                }
                let cells: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();

                if in_fundamental && cells.len() >= 3 {
                    if cells[0] != "Objective" {
                        fundamental.push(json!({
                            "objective": cells[0],
                            "measure": cells[1],
                            "direction": cells[2]
                        }));
                    }
                }

                if in_means && cells.len() >= 2 {
                    if cells[0] != "Means Objective" {
                        means.push(json!({
                            "objective": cells[0],
                            "supports": cells[1]
                        }));
                    }
                }
            }
        }

        let mut data = json!({});
        if !fundamental.is_empty() {
            data["fundamental"] = json!(fundamental);
        }
        if !means.is_empty() {
            data["means"] = json!(means);
        }

        if content.contains("Not yet started") {
            ParsedSection::success(ComponentType::Objectives, content.to_string(), json!({}))
        } else if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            errors.push(ParseError::warning(
                start_line,
                "Could not extract Objectives data",
            ));
            ParsedSection::with_errors(ComponentType::Objectives, content.to_string(), errors)
        } else {
            ParsedSection::success(ComponentType::Objectives, content.to_string(), data)
        }
    }

    /// Parses the Alternatives section.
    fn parse_alternatives(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut alternatives = Vec::new();
        let mut strategy_table = json!({});
        let mut errors = Vec::new();

        let mut in_options_table = false;
        let mut in_strategy_table = false;

        for (line_offset, line) in content.lines().enumerate() {
            let _line_num = start_line + line_offset;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                in_options_table = header.contains("Options");
                in_strategy_table = header.contains("Strategy");
                continue;
            }

            // Handle table rows
            if let Some(caps) = self.table_row_regex.captures(trimmed) {
                let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                if inner.contains("---") {
                    continue;
                }
                let cells: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();

                if in_options_table && cells.len() >= 4 {
                    if cells[0] != "#" {
                        let is_status_quo = cells[3].contains("Yes");
                        alternatives.push(json!({
                            "name": cells[1],
                            "description": cells[2],
                            "is_status_quo": is_status_quo
                        }));
                    }
                }

                if in_strategy_table && cells.len() >= 2 {
                    if cells[0] != "Sub-Decision" {
                        let sub_decisions = strategy_table
                            .get_mut("sub_decisions")
                            .and_then(|v| v.as_array_mut());
                        let options: Vec<&str> = cells[1..].iter().copied().collect();
                        let entry = json!({
                            "name": cells[0],
                            "options": options
                        });
                        if let Some(arr) = sub_decisions {
                            arr.push(entry);
                        } else {
                            strategy_table["sub_decisions"] = json!([entry]);
                        }
                    }
                }
            }
        }

        let mut data = json!({});
        if !alternatives.is_empty() {
            data["alternatives"] = json!(alternatives);
        }
        if strategy_table.get("sub_decisions").is_some() {
            data["strategy_table"] = strategy_table;
        }

        if content.contains("Not yet started") {
            ParsedSection::success(ComponentType::Alternatives, content.to_string(), json!({}))
        } else if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            errors.push(ParseError::warning(
                start_line,
                "Could not extract Alternatives data",
            ));
            ParsedSection::with_errors(ComponentType::Alternatives, content.to_string(), errors)
        } else {
            ParsedSection::success(ComponentType::Alternatives, content.to_string(), data)
        }
    }

    /// Parses the Consequences section (Pugh matrix).
    fn parse_consequences(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut matrix = json!({});
        let mut uncertainties = Vec::new();
        let mut errors = Vec::new();

        let mut in_matrix = false;
        let mut in_uncertainties = false;
        let mut alternatives: Vec<String> = Vec::new();
        let mut objectives: Vec<Value> = Vec::new();
        let mut totals: Vec<i64> = Vec::new();

        for (line_offset, line) in content.lines().enumerate() {
            let _line_num = start_line + line_offset;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                in_matrix = header.contains("Consequence Matrix") || header.contains("Pugh");
                in_uncertainties = header.contains("Uncertaint");
                continue;
            }

            // Handle table rows
            if let Some(caps) = self.table_row_regex.captures(trimmed) {
                let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                if inner.contains("---") || inner.contains(":---") {
                    continue;
                }
                let cells: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();

                if in_matrix {
                    if cells[0] == "Objective" {
                        // Header row - extract alternatives
                        alternatives = cells[1..].iter().map(|s| s.to_string()).collect();
                    } else if cells[0].contains("Total") {
                        // Total row
                        for cell in cells[1..].iter() {
                            let clean = cell
                                .replace("**", "")
                                .replace("+", "")
                                .trim()
                                .to_string();
                            totals.push(clean.parse().unwrap_or(0));
                        }
                    } else {
                        // Objective row
                        let scores: Vec<i64> = cells[1..]
                            .iter()
                            .map(|s| {
                                let clean = s.replace("+", "").trim().to_string();
                                clean.parse().unwrap_or(0)
                            })
                            .collect();
                        objectives.push(json!({
                            "name": cells[0],
                            "scores": scores
                        }));
                    }
                }

                if in_uncertainties && cells.len() >= 3 {
                    if cells[0] != "Uncertainty" {
                        let resolvable = cells[2].contains("Yes");
                        uncertainties.push(json!({
                            "description": cells[0],
                            "impact": cells[1],
                            "resolvable": resolvable
                        }));
                    }
                }
            }
        }

        let mut data = json!({});

        if !alternatives.is_empty() {
            let alt_json: Vec<Value> = alternatives
                .iter()
                .map(|name| json!({"name": name}))
                .collect();
            matrix["alternatives"] = json!(alt_json);
            matrix["objectives"] = json!(objectives);
            if !totals.is_empty() {
                matrix["totals"] = json!(totals);
            }
            data["consequence_matrix"] = matrix;
        }

        if !uncertainties.is_empty() {
            data["uncertainties"] = json!(uncertainties);
        }

        if content.contains("Not yet started") {
            ParsedSection::success(ComponentType::Consequences, content.to_string(), json!({}))
        } else if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            errors.push(ParseError::warning(
                start_line,
                "Could not extract Consequences data",
            ));
            ParsedSection::with_errors(ComponentType::Consequences, content.to_string(), errors)
        } else {
            ParsedSection::success(ComponentType::Consequences, content.to_string(), data)
        }
    }

    /// Parses the Tradeoffs section.
    fn parse_tradeoffs(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut dominated_alternatives: Vec<Value> = Vec::new();
        let mut irrelevant_objectives: Vec<String> = Vec::new();
        let mut key_tensions: Vec<Value> = Vec::new();

        let mut in_dominated = false;
        let mut in_irrelevant = false;
        let mut in_tensions = false;

        for (line_offset, line) in content.lines().enumerate() {
            let _line_num = start_line + line_offset;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                in_dominated = header.contains("Dominated");
                in_irrelevant = header.contains("Irrelevant");
                in_tensions = header.contains("Tension");
                continue;
            }

            // Handle key-value in lists (for dominated alternatives)
            if in_dominated {
                if let Some(caps) = self.key_value_regex.captures(trimmed.trim_start_matches('-').trim()) {
                    let name = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                    let reason = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
                    if !name.contains("None") {
                        dominated_alternatives.push(json!({
                            "name": name,
                            "reason": reason
                        }));
                    }
                }
            }

            // Handle list items for irrelevant objectives
            if in_irrelevant {
                if let Some(caps) = self.key_value_regex.captures(trimmed.trim_start_matches('-').trim()) {
                    let name = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                    if !name.contains("None") {
                        irrelevant_objectives.push(name.to_string());
                    }
                }
            }

            // Handle table rows for tensions
            if in_tensions {
                if let Some(caps) = self.table_row_regex.captures(trimmed) {
                    let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    if inner.contains("---") {
                        continue;
                    }
                    let cells: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();

                    if cells.len() >= 3 && cells[0] != "Alternative" {
                        key_tensions.push(json!({
                            "alternative": cells[0],
                            "excels_at": cells[1],
                            "sacrifices": cells[2]
                        }));
                    }
                }
            }
        }

        let mut data = json!({});
        data["dominated_alternatives"] = json!(dominated_alternatives);
        data["irrelevant_objectives"] = json!(irrelevant_objectives);
        if !key_tensions.is_empty() {
            data["key_tensions"] = json!(key_tensions);
        }

        if content.contains("Not yet started") {
            ParsedSection::success(ComponentType::Tradeoffs, content.to_string(), json!({}))
        } else {
            ParsedSection::success(ComponentType::Tradeoffs, content.to_string(), data)
        }
    }

    /// Parses the Recommendation section.
    fn parse_recommendation(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut data = json!({});
        let mut errors = Vec::new();
        let mut key_considerations = Vec::new();
        let mut remaining_uncertainties = Vec::new();

        let mut in_synthesis = false;
        let mut in_standout = false;
        let mut in_considerations = false;
        let mut in_uncertainties = false;

        for (line_offset, line) in content.lines().enumerate() {
            let _line_num = start_line + line_offset;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                in_synthesis = header.contains("Synthesis");
                in_standout = header.contains("Stands Out");
                in_considerations = header.contains("Key Considerations");
                in_uncertainties = header.contains("Remaining Uncertaint");
                continue;
            }

            // Handle blockquotes (synthesis)
            if in_synthesis {
                if let Some(caps) = self.blockquote_regex.captures(trimmed) {
                    let text = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                    if !text.is_empty() {
                        data["synthesis"] = json!(text);
                    }
                }
            }

            // Handle standout option
            if in_standout {
                if let Some(caps) = self.key_value_regex.captures(trimmed.trim_start_matches('-').trim()) {
                    let key = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                    let value = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();

                    if key.contains("Standout Option") {
                        if data.get("standout_option").is_none() {
                            data["standout_option"] = json!({});
                        }
                        data["standout_option"]["name"] = json!(value);
                    } else if key.contains("Rationale") {
                        if data.get("standout_option").is_none() {
                            data["standout_option"] = json!({});
                        }
                        data["standout_option"]["rationale"] = json!(value);
                    }
                }
            }

            // Handle numbered list for considerations
            if in_considerations {
                if let Some(caps) = self.list_item_regex.captures(trimmed) {
                    let item = caps
                        .get(1)
                        .or_else(|| caps.get(3))
                        .map(|m| m.as_str())
                        .unwrap_or("")
                        .trim();
                    if !item.is_empty() {
                        key_considerations.push(item.to_string());
                    }
                }
            }

            // Handle remaining uncertainties
            if in_uncertainties {
                if let Some(caps) = self.list_item_regex.captures(trimmed) {
                    let item = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                    // Split on " - " for description and resolution
                    let parts: Vec<&str> = item.splitn(2, " - ").collect();
                    if parts.len() == 2 {
                        remaining_uncertainties.push(json!({
                            "description": parts[0].trim(),
                            "resolution_path": parts[1].trim()
                        }));
                    } else if !item.is_empty() {
                        remaining_uncertainties.push(json!({
                            "description": item,
                            "resolution_path": ""
                        }));
                    }
                }
            }
        }

        if !key_considerations.is_empty() {
            data["key_considerations"] = json!(key_considerations);
        }
        if !remaining_uncertainties.is_empty() {
            data["remaining_uncertainties"] = json!(remaining_uncertainties);
        }

        if content.contains("Not yet started") {
            ParsedSection::success(ComponentType::Recommendation, content.to_string(), json!({}))
        } else if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            errors.push(ParseError::warning(
                start_line,
                "Could not extract Recommendation data",
            ));
            ParsedSection::with_errors(ComponentType::Recommendation, content.to_string(), errors)
        } else {
            ParsedSection::success(ComponentType::Recommendation, content.to_string(), data)
        }
    }

    /// Parses the Decision Quality section.
    fn parse_decision_quality(&self, content: &str, start_line: usize) -> ParsedSection {
        let mut elements = Vec::new();
        let mut improvements = Vec::new();
        let mut errors = Vec::new();

        let mut in_improvements = false;

        for (line_offset, line) in content.lines().enumerate() {
            let _line_num = start_line + line_offset;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for subsection headers
            if let Some(caps) = self.subsection_header_regex.captures(trimmed) {
                let header = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                in_improvements = header.contains("Improve");
                continue;
            }

            // Handle table rows for element scores
            if !in_improvements {
                if let Some(caps) = self.table_row_regex.captures(trimmed) {
                    let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    if inner.contains("---") || inner.contains(":---") {
                        continue;
                    }
                    let cells: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();

                    if cells.len() >= 3 {
                        // Skip header row
                        if cells[0] == "Element" || cells[0].contains("**Overall") {
                            continue;
                        }
                        // Parse score (remove %)
                        let score_str = cells[1].replace("%", "").trim().to_string();
                        if let Ok(score) = score_str.parse::<u8>() {
                            elements.push(json!({
                                "name": cells[0],
                                "score": score,
                                "rationale": cells[2]
                            }));
                        }
                    }
                }
            }

            // Handle checkbox items for improvements
            if in_improvements {
                if let Some(caps) = self.checkbox_regex.captures(trimmed) {
                    let item = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
                    // Parse "action (improvement)"
                    if let Some(paren_start) = item.find('(') {
                        let action = item[..paren_start].trim();
                        let improvement = item[paren_start + 1..]
                            .trim_end_matches(')')
                            .trim();
                        improvements.push(json!({
                            "action": action,
                            "improvement": improvement
                        }));
                    }
                }
            }
        }

        let mut data = json!({});
        if !elements.is_empty() {
            data["elements"] = json!(elements);
        }
        if !improvements.is_empty() {
            data["improvements"] = json!(improvements);
        }

        if content.contains("Not yet started") {
            ParsedSection::success(
                ComponentType::DecisionQuality,
                content.to_string(),
                json!({}),
            )
        } else if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            errors.push(ParseError::warning(
                start_line,
                "Could not extract Decision Quality data",
            ));
            ParsedSection::with_errors(
                ComponentType::DecisionQuality,
                content.to_string(),
                errors,
            )
        } else {
            ParsedSection::success(ComponentType::DecisionQuality, content.to_string(), data)
        }
    }

    /// Parses a section based on its component type.
    fn parse_section_by_type(
        &self,
        component_type: ComponentType,
        content: &str,
        start_line: usize,
    ) -> ParsedSection {
        match component_type {
            ComponentType::IssueRaising => self.parse_issue_raising(content, start_line),
            ComponentType::ProblemFrame => self.parse_problem_frame(content, start_line),
            ComponentType::Objectives => self.parse_objectives(content, start_line),
            ComponentType::Alternatives => self.parse_alternatives(content, start_line),
            ComponentType::Consequences => self.parse_consequences(content, start_line),
            ComponentType::Tradeoffs => self.parse_tradeoffs(content, start_line),
            ComponentType::Recommendation => self.parse_recommendation(content, start_line),
            ComponentType::DecisionQuality => self.parse_decision_quality(content, start_line),
            ComponentType::NotesNextSteps => {
                // Not part of the 8 main sections, return empty
                ParsedSection::success(component_type, content.to_string(), json!({}))
            }
        }
    }
}

impl DocumentParser for MarkdownDocumentParser {
    fn parse(&self, content: &str) -> Result<ParseResult, DocumentError> {
        let mut result = ParseResult::empty();

        // Extract metadata from header
        result.metadata = self.extract_metadata(content);

        // Extract section boundaries
        let boundaries = self.extract_section_boundaries(content);

        // Parse each section
        let lines: Vec<&str> = content.lines().collect();

        for boundary in &boundaries {
            let section_lines = &lines[boundary.start_line - 1..boundary.end_line];
            let section_content = section_lines.join("\n");

            let parsed_section =
                self.parse_section_by_type(boundary.component_type, &section_content, boundary.start_line);

            result.sections.push(parsed_section);
        }

        Ok(result)
    }

    fn parse_section(
        &self,
        section_content: &str,
        expected_type: ComponentType,
    ) -> Result<ParsedSection, DocumentError> {
        Ok(self.parse_section_by_type(expected_type, section_content, 1))
    }

    fn validate_structure(&self, content: &str) -> Result<Vec<ParseError>, DocumentError> {
        let mut errors = Vec::new();

        // Check for title
        if self.extract_title(content).is_none() {
            errors.push(ParseError::warning(1, "Missing document title (# heading)"));
        }

        // Check for expected sections
        let boundaries = self.extract_section_boundaries(content);
        let found_types: Vec<ComponentType> = boundaries.iter().map(|b| b.component_type).collect();

        let expected = [
            ComponentType::IssueRaising,
            ComponentType::ProblemFrame,
            ComponentType::Objectives,
            ComponentType::Alternatives,
            ComponentType::Consequences,
            ComponentType::Tradeoffs,
            ComponentType::Recommendation,
            ComponentType::DecisionQuality,
        ];

        for expected_type in expected {
            if !found_types.contains(&expected_type) {
                errors.push(ParseError::warning(
                    1,
                    format!("Missing section: {}", section_name(expected_type)),
                ));
            }
        }

        // Check section order
        let mut last_order = 0;
        for boundary in &boundaries {
            let order = component_order(boundary.component_type);
            if order < last_order {
                errors.push(ParseError::warning(
                    boundary.start_line,
                    format!(
                        "Section {} is out of order",
                        section_name(boundary.component_type)
                    ),
                ));
            }
            last_order = order;
        }

        Ok(errors)
    }

    fn extract_section_boundaries(&self, content: &str) -> Vec<SectionBoundary> {
        let mut boundaries = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut current_boundary: Option<SectionBoundary> = None;

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            // Check for section header
            if let Some(caps) = self.section_header_regex.captures(trimmed) {
                // Close previous boundary
                if let Some(mut boundary) = current_boundary.take() {
                    boundary.end_line = line_num - 1;
                    // Don't add empty boundaries
                    if boundary.end_line >= boundary.start_line {
                        boundaries.push(boundary);
                    }
                }

                // Start new boundary
                if let Some(number_str) = caps.get(1) {
                    if let Ok(number) = number_str.as_str().parse::<u32>() {
                        if let Some(component_type) = Self::section_number_to_type(number) {
                            let heading = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                            current_boundary = Some(SectionBoundary::new(
                                component_type,
                                line_num,
                                line_num, // Will be updated
                                heading,
                            ));
                        }
                    }
                }
            }
        }

        // Close final boundary
        if let Some(mut boundary) = current_boundary.take() {
            boundary.end_line = lines.len();
            boundaries.push(boundary);
        }

        boundaries
    }
}

/// Returns the section name for a component type.
fn section_name(component_type: ComponentType) -> &'static str {
    match component_type {
        ComponentType::IssueRaising => "Issue Raising",
        ComponentType::ProblemFrame => "Problem Frame",
        ComponentType::Objectives => "Objectives",
        ComponentType::Alternatives => "Alternatives",
        ComponentType::Consequences => "Consequences",
        ComponentType::Tradeoffs => "Tradeoffs",
        ComponentType::Recommendation => "Recommendation",
        ComponentType::DecisionQuality => "Decision Quality",
        ComponentType::NotesNextSteps => "Notes & Next Steps",
    }
}

/// Returns the expected order for a component type.
fn component_order(component_type: ComponentType) -> u32 {
    match component_type {
        ComponentType::IssueRaising => 1,
        ComponentType::ProblemFrame => 2,
        ComponentType::Objectives => 3,
        ComponentType::Alternatives => 4,
        ComponentType::Consequences => 5,
        ComponentType::Tradeoffs => 6,
        ComponentType::Recommendation => 7,
        ComponentType::DecisionQuality => 8,
        ComponentType::NotesNextSteps => 9,
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parser() -> MarkdownDocumentParser {
        MarkdownDocumentParser::new()
    }

    // ───────────────────────────────────────────────────────────────
    // Section Boundary Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn extract_boundaries_from_document() {
        let parser = test_parser();
        let content = r#"# My Decision

## 1. Issue Raising

Some content here.

## 2. Problem Frame

More content.

## 3. Objectives

Even more content.
"#;

        let boundaries = parser.extract_section_boundaries(content);

        assert_eq!(boundaries.len(), 3);
        assert_eq!(boundaries[0].component_type, ComponentType::IssueRaising);
        assert_eq!(boundaries[0].start_line, 3);
        assert_eq!(boundaries[1].component_type, ComponentType::ProblemFrame);
        assert_eq!(boundaries[2].component_type, ComponentType::Objectives);
    }

    // ───────────────────────────────────────────────────────────────
    // Issue Raising Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_issue_raising_with_data() {
        let parser = test_parser();
        let content = r#"## 1. Issue Raising

> A career transition decision

### Potential Decisions
- [ ] Accept new job
- [ ] Stay current

### Objectives Identified
- Better compensation
- Work-life balance

### Uncertainties
- Market conditions

### Considerations
- Family impact
"#;

        let section = parser.parse_issue_raising(content, 1);

        assert!(section.is_success());
        let data = section.parsed_data.unwrap();
        // Domain-compatible output (matches IssueRaisingOutput)
        assert_eq!(data["potential_decisions"].as_array().unwrap().len(), 2);
        assert_eq!(data["objectives"].as_array().unwrap().len(), 2);
        assert_eq!(data["uncertainties"].as_array().unwrap().len(), 1);
        assert_eq!(data["considerations"].as_array().unwrap().len(), 1);
        assert_eq!(data["user_confirmed"], false);
    }

    #[test]
    fn parse_issue_raising_empty() {
        let parser = test_parser();
        let content = "## 1. Issue Raising\n\n*Not yet started*\n";

        let section = parser.parse_issue_raising(content, 1);

        assert!(section.is_success());
        let data = section.parsed_data.unwrap();
        assert!(data.as_object().unwrap().is_empty());
    }

    // ───────────────────────────────────────────────────────────────
    // Problem Frame Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_problem_frame_with_data() {
        let parser = test_parser();
        let content = r#"## 2. Problem Frame

**Decision Maker:** Alice (Manager)

**Focal Decision:**
> Should I accept the VP offer?

**Scope:** Career decision only

**Deadline:** End of month
"#;

        let section = parser.parse_problem_frame(content, 1);

        assert!(section.is_success());
        let data = section.parsed_data.unwrap();
        assert_eq!(data["decision_maker"]["name"], "Alice");
        assert_eq!(data["decision_maker"]["role"], "Manager");
        assert_eq!(data["focal_decision"], "Should I accept the VP offer?");
        assert_eq!(data["scope"], "Career decision only");
        assert_eq!(data["deadline"], "End of month");
    }

    // ───────────────────────────────────────────────────────────────
    // Objectives Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_objectives_with_tables() {
        let parser = test_parser();
        let content = r#"## 3. Objectives

### Fundamental Objectives (What Really Matters)

| Objective | Measure | Direction |
|-----------|---------|----------|
| Maximize income | $/year | ↑ Higher is better |
| Work-life balance | Hours/week | ↓ Lower is better |

### Means Objectives (Ways to Achieve)

| Means Objective | Supports |
|-----------------|----------|
| Reduce commute | Work-life balance |
"#;

        let section = parser.parse_objectives(content, 1);

        assert!(section.is_success());
        let data = section.parsed_data.unwrap();

        let fundamental = data["fundamental"].as_array().unwrap();
        assert_eq!(fundamental.len(), 2);
        assert_eq!(fundamental[0]["objective"], "Maximize income");
        assert_eq!(fundamental[0]["measure"], "$/year");

        let means = data["means"].as_array().unwrap();
        assert_eq!(means.len(), 1);
        assert_eq!(means[0]["objective"], "Reduce commute");
    }

    // ───────────────────────────────────────────────────────────────
    // Alternatives Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_alternatives_with_data() {
        let parser = test_parser();
        let content = r#"## 4. Alternatives

### Options Under Consideration

| # | Alternative | Description | Status Quo? |
|---|-------------|-------------|-------------|
| A | Accept VP | Take the new role | No |
| B | Stay current | Keep current position | **Yes** |
"#;

        let section = parser.parse_alternatives(content, 1);

        assert!(section.is_success());
        let data = section.parsed_data.unwrap();

        let alternatives = data["alternatives"].as_array().unwrap();
        assert_eq!(alternatives.len(), 2);
        assert_eq!(alternatives[0]["name"], "Accept VP");
        assert!(!alternatives[0]["is_status_quo"].as_bool().unwrap());
        assert!(alternatives[1]["is_status_quo"].as_bool().unwrap());
    }

    // ───────────────────────────────────────────────────────────────
    // Consequences Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_consequences_pugh_matrix() {
        let parser = test_parser();
        let content = r#"## 5. Consequences

### Consequence Matrix (Pugh Analysis)

| Objective | A: Accept | B: Stay |
|-----------|:---:|:---:|
| Income | +2 | 0 |
| Balance | -1 | 0 |
| **Total** | **+1** | 0 |

**Rating Scale:** -2 (Much Worse) → -1 (Worse) → 0 (Same) → +1 (Better) → +2 (Much Better)
"#;

        let section = parser.parse_consequences(content, 1);

        assert!(section.is_success());
        let data = section.parsed_data.unwrap();

        let matrix = &data["consequence_matrix"];
        let objectives = matrix["objectives"].as_array().unwrap();
        assert_eq!(objectives.len(), 2);
        assert_eq!(objectives[0]["name"], "Income");
        assert_eq!(objectives[0]["scores"][0], 2);
        assert_eq!(objectives[0]["scores"][1], 0);

        let totals = matrix["totals"].as_array().unwrap();
        assert_eq!(totals[0], 1);
        assert_eq!(totals[1], 0);
    }

    // ───────────────────────────────────────────────────────────────
    // Decision Quality Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_decision_quality_scores() {
        let parser = test_parser();
        let content = r#"## 8. Decision Quality Assessment

| Element | Score | Rationale |
|---------|:-----:|----------|
| Clear Problem Frame | 85% | Well defined |
| Clear Objectives | 70% | Needs measures |
| **Overall Quality** | **70%** | *Weakest link: Clear Objectives* |

### To Improve Quality
- [ ] Add measures to objectives (Would improve DQ by 10%)
"#;

        let section = parser.parse_decision_quality(content, 1);

        assert!(section.is_success());
        let data = section.parsed_data.unwrap();

        let elements = data["elements"].as_array().unwrap();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0]["name"], "Clear Problem Frame");
        assert_eq!(elements[0]["score"], 85);
        assert_eq!(elements[1]["score"], 70);

        let improvements = data["improvements"].as_array().unwrap();
        assert_eq!(improvements.len(), 1);
        assert_eq!(improvements[0]["action"], "Add measures to objectives");
    }

    // ───────────────────────────────────────────────────────────────
    // Full Document Parse Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_full_document() {
        let parser = test_parser();
        let content = r#"# Career Decision

> **Status:** In Progress | **Quality Score:** --

---

## 1. Issue Raising

> A career transition decision

---

## 2. Problem Frame

**Decision Maker:** Alice (Manager)

---

## 3. Objectives

*Not yet started*

---
"#;

        let result = parser.parse(content).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.metadata.title, Some("Career Decision".to_string()));
        assert_eq!(result.metadata.status, Some("In Progress".to_string()));
        assert_eq!(result.sections.len(), 3);
        assert_eq!(
            result.sections[0].component_type,
            ComponentType::IssueRaising
        );
    }

    // ───────────────────────────────────────────────────────────────
    // Structure Validation Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn validate_structure_missing_sections() {
        let parser = test_parser();
        let content = r#"# My Decision

## 1. Issue Raising

Content
"#;

        let errors = parser.validate_structure(content).unwrap();

        // Should warn about missing sections 2-8
        assert!(errors.len() >= 7);
    }

    #[test]
    fn validate_structure_complete_document() {
        let parser = test_parser();
        let content = r#"# My Decision

## 1. Issue Raising
## 2. Problem Frame
## 3. Objectives
## 4. Alternatives
## 5. Consequences
## 6. Tradeoffs
## 7. Recommendation
## 8. Decision Quality Assessment
"#;

        let errors = parser.validate_structure(content).unwrap();

        assert!(errors.is_empty());
    }
}
