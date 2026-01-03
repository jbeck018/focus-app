// ai/tool_parser.rs - Parse tool calls from LLM output
//
// This module handles parsing tool calls from LLM-generated text.
// It supports multiple formats to be flexible with LLM output variations.
//
// Supported formats:
// 1. XML-style: <tool name="start_focus_session" duration="25"/>
// 2. Function-style: [[start_focus_session(duration=25)]]
// 3. JSON in markdown: ```json\n{"tool": "start_focus_session", "params": {"duration": 25}}\n```
// 4. Bracket-style: [start_focus_session:25] or [start_focus_session]

use crate::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A parsed tool call
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedToolCall {
    /// The name of the tool to invoke
    pub name: String,
    /// Parameters for the tool
    pub params: serde_json::Value,
    /// The original text that was parsed
    pub raw_text: String,
    /// Start position in the original string
    pub start_pos: usize,
    /// End position in the original string
    pub end_pos: usize,
}

impl ParsedToolCall {
    /// Create a new parsed tool call
    #[allow(dead_code)] // Public API method - used by tests and parser internals
    pub fn new(
        name: impl Into<String>,
        params: serde_json::Value,
        raw_text: impl Into<String>,
        start_pos: usize,
        end_pos: usize,
    ) -> Self {
        Self {
            name: name.into(),
            params,
            raw_text: raw_text.into(),
            start_pos,
            end_pos,
        }
    }

    /// Get a parameter value as a string
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.params.get(key).and_then(|v| v.as_str())
    }

    /// Get a parameter value as an integer
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.params.get(key).and_then(|v| v.as_i64())
    }

    /// Get a parameter value as a boolean
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.params.get(key).and_then(|v| v.as_bool())
    }
}

/// Result of parsing LLM output
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Tool calls found in the text
    pub tool_calls: Vec<ParsedToolCall>,
    /// Text segments between tool calls (for display)
    pub text_segments: Vec<TextSegment>,
    /// Whether any tool calls were found
    pub has_tools: bool,
}

/// A segment of text (either plain text or a tool call reference)
#[derive(Debug, Clone)]
#[allow(dead_code)] // All variants used by parser
pub enum TextSegment {
    /// Plain text content
    Text(String),
    /// Reference to a tool call by index
    ToolCall(usize),
}

impl ParseResult {
    /// Create an empty parse result
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn empty() -> Self {
        Self {
            tool_calls: Vec::new(),
            text_segments: Vec::new(),
            has_tools: false,
        }
    }

    /// Create a result with only text (no tool calls)
    #[allow(dead_code)] // Used by parser when no tools found
    pub fn text_only(text: String) -> Self {
        Self {
            tool_calls: Vec::new(),
            text_segments: vec![TextSegment::Text(text)],
            has_tools: false,
        }
    }

    /// Get the text content without tool calls
    pub fn text_without_tools(&self) -> String {
        self.text_segments
            .iter()
            .filter_map(|seg| match seg {
                TextSegment::Text(t) => Some(t.as_str()),
                TextSegment::ToolCall(_) => None,
            })
            .collect::<Vec<_>>()
            .join(" ")  // Add space between segments for proper text flow
            .split_whitespace()  // Normalize whitespace
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get the first tool call if any
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn first_tool(&self) -> Option<&ParsedToolCall> {
        self.tool_calls.first()
    }
}

/// Parser for tool calls in LLM output
pub struct ToolParser {
    /// Regex for XML-style tool calls: <tool name="..." .../>
    xml_regex: Regex,
    /// Regex for function-style calls: [[tool_name(param=value)]]
    function_regex: Regex,
    /// Regex for JSON in markdown blocks
    json_block_regex: Regex,
    /// Regex for bracket-style calls: [tool_name:param] or [tool_name]
    bracket_regex: Regex,
}

impl ToolParser {
    /// Create a new tool parser
    pub fn new() -> Self {
        Self {
            // Match <tool name="..." /> or <tool name="...">...</tool>
            xml_regex: Regex::new(
                r#"<tool\s+name\s*=\s*"([^"]+)"([^/>]*?)(?:/>|>\s*</tool>)"#
            ).expect("Invalid XML regex"),

            // Match [[tool_name(param1=value1, param2="value2")]]
            function_regex: Regex::new(
                r#"\[\[([a-z_]+)\(([^)]*)\)\]\]"#
            ).expect("Invalid function regex"),

            // Match ```json\n{...}\n``` blocks
            json_block_regex: Regex::new(
                r#"```json\s*\n?\s*(\{[^`]*\})\s*\n?```"#
            ).expect("Invalid JSON block regex"),

            // Match [tool_name] or [tool_name:param] or [tool_name:param1:param2]
            // Allows single or multiple colon-separated parameters
            bracket_regex: Regex::new(
                r#"\[([a-z_]+)(?::([^\]]*))?\]"#
            ).expect("Invalid bracket regex"),
        }
    }

    /// Parse LLM output for tool calls
    ///
    /// Attempts to parse using all supported formats and returns all found tool calls.
    pub fn parse(&self, text: &str) -> ParseResult {
        let mut all_calls: Vec<(ParsedToolCall, usize, usize)> = Vec::new();

        // Try XML format
        for cap in self.xml_regex.captures_iter(text) {
            if let Some(call) = self.parse_xml_capture(&cap, text) {
                let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
                let end = cap.get(0).map(|m| m.end()).unwrap_or(0);
                all_calls.push((call, start, end));
            }
        }

        // Try function format
        for cap in self.function_regex.captures_iter(text) {
            if let Some(call) = self.parse_function_capture(&cap, text) {
                let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
                let end = cap.get(0).map(|m| m.end()).unwrap_or(0);
                all_calls.push((call, start, end));
            }
        }

        // Try JSON block format
        for cap in self.json_block_regex.captures_iter(text) {
            if let Some(call) = self.parse_json_capture(&cap, text) {
                let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
                let end = cap.get(0).map(|m| m.end()).unwrap_or(0);
                all_calls.push((call, start, end));
            }
        }

        // Try bracket format [tool_name:param] or [tool_name]
        for cap in self.bracket_regex.captures_iter(text) {
            if let Some(call) = self.parse_bracket_capture(&cap, text) {
                let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
                let end = cap.get(0).map(|m| m.end()).unwrap_or(0);
                all_calls.push((call, start, end));
            }
        }

        if all_calls.is_empty() {
            return ParseResult::text_only(text.to_string());
        }

        // Sort by position
        all_calls.sort_by_key(|(_, start, _)| *start);

        // Build text segments
        let mut segments = Vec::new();
        let mut last_end = 0;

        let tool_calls: Vec<ParsedToolCall> = all_calls
            .into_iter()
            .enumerate()
            .map(|(idx, (call, start, end))| {
                // Add text before this tool call
                if start > last_end {
                    let text_before = text[last_end..start].trim();
                    if !text_before.is_empty() {
                        segments.push(TextSegment::Text(text_before.to_string()));
                    }
                }

                // Add tool call reference
                segments.push(TextSegment::ToolCall(idx));
                last_end = end;

                call
            })
            .collect();

        // Add remaining text
        if last_end < text.len() {
            let remaining = text[last_end..].trim();
            if !remaining.is_empty() {
                segments.push(TextSegment::Text(remaining.to_string()));
            }
        }

        ParseResult {
            has_tools: !tool_calls.is_empty(),
            tool_calls,
            text_segments: segments,
        }
    }

    /// Parse a single tool call string (for testing/direct use)
    #[allow(dead_code)] // Public API - may be used by external callers
    pub fn parse_single(&self, text: &str) -> Option<ParsedToolCall> {
        let result = self.parse(text);
        result.tool_calls.into_iter().next()
    }

    /// Parse XML-style capture
    fn parse_xml_capture(
        &self,
        cap: &regex::Captures,
        _full_text: &str,
    ) -> Option<ParsedToolCall> {
        let name = cap.get(1)?.as_str();
        let attrs_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let full_match = cap.get(0)?;

        let params = self.parse_xml_attributes(attrs_str);

        Some(ParsedToolCall::new(
            name,
            params,
            full_match.as_str(),
            full_match.start(),
            full_match.end(),
        ))
    }

    /// Parse XML attributes into a JSON object
    fn parse_xml_attributes(&self, attrs_str: &str) -> serde_json::Value {
        let mut params = serde_json::Map::new();

        // Match attribute="value" or attribute='value' or attribute=value
        let attr_regex = Regex::new(r#"(\w+)\s*=\s*(?:"([^"]*)"|'([^']*)'|(\S+))"#)
            .expect("Invalid attr regex");

        for cap in attr_regex.captures_iter(attrs_str) {
            if let Some(key) = cap.get(1) {
                // Value can be in group 2, 3, or 4 depending on quote style
                let value = cap.get(2)
                    .or_else(|| cap.get(3))
                    .or_else(|| cap.get(4))
                    .map(|m| m.as_str())
                    .unwrap_or("");

                // Try to parse as number or boolean
                let json_value = if let Ok(n) = value.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else if let Ok(f) = value.parse::<f64>() {
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
                } else if value == "true" {
                    serde_json::Value::Bool(true)
                } else if value == "false" {
                    serde_json::Value::Bool(false)
                } else {
                    serde_json::Value::String(value.to_string())
                };

                params.insert(key.as_str().to_string(), json_value);
            }
        }

        serde_json::Value::Object(params)
    }

    /// Parse function-style capture
    fn parse_function_capture(
        &self,
        cap: &regex::Captures,
        _full_text: &str,
    ) -> Option<ParsedToolCall> {
        let name = cap.get(1)?.as_str();
        let args_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let full_match = cap.get(0)?;

        let params = self.parse_function_args(args_str);

        Some(ParsedToolCall::new(
            name,
            params,
            full_match.as_str(),
            full_match.start(),
            full_match.end(),
        ))
    }

    /// Parse function arguments (key=value format)
    fn parse_function_args(&self, args_str: &str) -> serde_json::Value {
        let mut params = serde_json::Map::new();

        if args_str.trim().is_empty() {
            return serde_json::Value::Object(params);
        }

        // Split by comma, handling quoted values
        let arg_regex = Regex::new(r#"(\w+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^,\s]+))"#)
            .expect("Invalid arg regex");

        for cap in arg_regex.captures_iter(args_str) {
            if let Some(key) = cap.get(1) {
                let value = cap.get(2)
                    .or_else(|| cap.get(3))
                    .or_else(|| cap.get(4))
                    .map(|m| m.as_str())
                    .unwrap_or("");

                // Try to parse as number or boolean
                let json_value = if let Ok(n) = value.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else if let Ok(f) = value.parse::<f64>() {
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
                } else if value == "true" {
                    serde_json::Value::Bool(true)
                } else if value == "false" {
                    serde_json::Value::Bool(false)
                } else {
                    serde_json::Value::String(value.to_string())
                };

                params.insert(key.as_str().to_string(), json_value);
            }
        }

        serde_json::Value::Object(params)
    }

    /// Parse JSON block capture
    fn parse_json_capture(
        &self,
        cap: &regex::Captures,
        _full_text: &str,
    ) -> Option<ParsedToolCall> {
        let json_str = cap.get(1)?.as_str();
        let full_match = cap.get(0)?;

        // Parse the JSON
        let json: serde_json::Value = serde_json::from_str(json_str).ok()?;

        // Extract tool name and params
        let name = json.get("tool")
            .or_else(|| json.get("name"))
            .and_then(|v| v.as_str())?;

        let params = json.get("params")
            .or_else(|| json.get("parameters"))
            .or_else(|| json.get("args"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        Some(ParsedToolCall::new(
            name,
            params,
            full_match.as_str(),
            full_match.start(),
            full_match.end(),
        ))
    }

    /// Parse bracket-style capture [tool_name:param] or [tool_name]
    fn parse_bracket_capture(
        &self,
        cap: &regex::Captures,
        _full_text: &str,
    ) -> Option<ParsedToolCall> {
        let name = cap.get(1)?.as_str();
        let full_match = cap.get(0)?;

        // Get the parameter string if present
        let params_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        // Parse parameters from the bracket format
        // Format can be: [tool] or [tool:15] or [tool:15:true] or [tool:duration=15]
        let params = self.parse_bracket_params(params_str, name);

        Some(ParsedToolCall::new(
            name,
            params,
            full_match.as_str(),
            full_match.start(),
            full_match.end(),
        ))
    }

    /// Parse bracket-style parameters
    /// Handles formats like:
    /// - "15" -> {"duration": 15} for duration-based tools
    /// - "true" -> {"completed": true}
    /// - "duration=15" -> {"duration": 15}
    fn parse_bracket_params(&self, params_str: &str, tool_name: &str) -> serde_json::Value {
        let mut params = serde_json::Map::new();

        if params_str.trim().is_empty() {
            return serde_json::Value::Object(params);
        }

        // Check if it looks like key=value format
        if params_str.contains('=') {
            // Parse key=value format
            let arg_regex = Regex::new(r#"(\w+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^,\s]+))"#)
                .expect("Invalid arg regex");

            for cap in arg_regex.captures_iter(params_str) {
                if let Some(key) = cap.get(1) {
                    let value = cap.get(2)
                        .or_else(|| cap.get(3))
                        .or_else(|| cap.get(4))
                        .map(|m| m.as_str())
                        .unwrap_or("");

                    // Try to parse as number or boolean
                    let json_value = if let Ok(n) = value.parse::<i64>() {
                        serde_json::Value::Number(n.into())
                    } else if let Ok(f) = value.parse::<f64>() {
                        serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
                    } else if value == "true" {
                        serde_json::Value::Bool(true)
                    } else if value == "false" {
                        serde_json::Value::Bool(false)
                    } else {
                        serde_json::Value::String(value.to_string())
                    };

                    params.insert(key.as_str().to_string(), json_value);
                }
            }
        } else {
            // Parse positional parameters based on tool type
            // For duration-based tools, treat first param as duration
            if tool_name.contains("session") || tool_name.contains("goal") {
                if let Ok(duration) = params_str.trim().parse::<i64>() {
                    params.insert("duration".to_string(), serde_json::Value::Number(duration.into()));
                } else if params_str == "true" || params_str == "false" {
                    let bool_val = params_str == "true";
                    params.insert("completed".to_string(), serde_json::Value::Bool(bool_val));
                } else {
                    // Treat as string parameter
                    params.insert("value".to_string(), serde_json::Value::String(params_str.to_string()));
                }
            } else if params_str == "true" || params_str == "false" {
                // Boolean value
                let bool_val = params_str == "true";
                params.insert("completed".to_string(), serde_json::Value::Bool(bool_val));
            } else if let Ok(num) = params_str.trim().parse::<i64>() {
                // Numeric value - use as first available param
                params.insert("value".to_string(), serde_json::Value::Number(num.into()));
            } else {
                // String value
                params.insert("value".to_string(), serde_json::Value::String(params_str.to_string()));
            }
        }

        serde_json::Value::Object(params)
    }
}

impl Default for ToolParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate that a tool call has required parameters
///
/// Public API utility - may be used by external callers for parameter validation
#[allow(dead_code)]
pub fn validate_tool_call(
    call: &ParsedToolCall,
    required_params: &[&str],
) -> Result<()> {
    for param in required_params {
        if call.params.get(*param).is_none() {
            return Err(Error::InvalidInput(format!(
                "Missing required parameter '{}' for tool '{}'",
                param, call.name
            )));
        }
    }
    Ok(())
}

/// Check if text contains any tool calls
///
/// Public API utility - may be used by external callers
#[allow(dead_code)]
pub fn contains_tool_calls(text: &str) -> bool {
    let parser = ToolParser::new();
    parser.parse(text).has_tools
}

/// Extract the first tool call from text
///
/// Public API utility - may be used by external callers
#[allow(dead_code)]
pub fn extract_first_tool(text: &str) -> Option<ParsedToolCall> {
    let parser = ToolParser::new();
    parser.parse(text).tool_calls.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xml_simple() {
        let parser = ToolParser::new();
        let result = parser.parse(r#"<tool name="start_focus_session"/>"#);

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
    }

    #[test]
    fn test_parse_xml_with_params() {
        let parser = ToolParser::new();
        let result = parser.parse(
            r#"<tool name="start_focus_session" duration="45" strict="true"/>"#
        );

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
        assert_eq!(result.tool_calls[0].get_int("duration"), Some(45));
        assert_eq!(result.tool_calls[0].get_bool("strict"), Some(true));
    }

    #[test]
    fn test_parse_xml_in_text() {
        let parser = ToolParser::new();
        let text = r#"Sure, I'll start a session for you.

<tool name="start_focus_session" duration="25"/>

Let me know if you need anything else!"#;

        let result = parser.parse(text);

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.text_segments.len(), 3);

        let text_only = result.text_without_tools();
        assert!(text_only.contains("Sure"));
        assert!(text_only.contains("anything else"));
    }

    #[test]
    fn test_parse_function_style() {
        let parser = ToolParser::new();
        let result = parser.parse(r#"[[start_focus_session(duration=45)]]"#);

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
        assert_eq!(result.tool_calls[0].get_int("duration"), Some(45));
    }

    #[test]
    fn test_parse_function_with_string_param() {
        let parser = ToolParser::new();
        let result = parser.parse(
            r#"[[log_trigger(trigger_type="boredom", notes="test note")]]"#
        );

        assert!(result.has_tools);
        assert_eq!(result.tool_calls[0].name, "log_trigger");
        assert_eq!(result.tool_calls[0].get_string("trigger_type"), Some("boredom"));
        assert_eq!(result.tool_calls[0].get_string("notes"), Some("test note"));
    }

    #[test]
    fn test_parse_json_block() {
        let parser = ToolParser::new();
        let result = parser.parse(
            r#"```json
{"tool": "start_focus_session", "params": {"duration": 30}}
```"#
        );

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
        assert_eq!(result.tool_calls[0].get_int("duration"), Some(30));
    }

    #[test]
    fn test_parse_multiple_tools() {
        let parser = ToolParser::new();
        let text = r#"I'll help you with that.

<tool name="get_session_stats" period="today"/>

Based on those stats, let me also check your streak:

<tool name="get_streak_info"/>"#;

        let result = parser.parse(text);

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 2);
        assert_eq!(result.tool_calls[0].name, "get_session_stats");
        assert_eq!(result.tool_calls[1].name, "get_streak_info");
    }

    #[test]
    fn test_parse_no_tools() {
        let parser = ToolParser::new();
        let result = parser.parse("This is just regular text with no tool calls.");

        assert!(!result.has_tools);
        assert!(result.tool_calls.is_empty());
    }

    #[test]
    fn test_validate_tool_call() {
        let call = ParsedToolCall::new(
            "log_trigger",
            serde_json::json!({"trigger_type": "boredom"}),
            "",
            0,
            0,
        );

        assert!(validate_tool_call(&call, &["trigger_type"]).is_ok());
        assert!(validate_tool_call(&call, &["trigger_type", "notes"]).is_err());
    }

    #[test]
    fn test_contains_tool_calls() {
        assert!(contains_tool_calls(r#"<tool name="test"/>"#));
        assert!(contains_tool_calls(r#"[[test()]]"#));
        assert!(!contains_tool_calls("No tools here"));
    }

    #[test]
    fn test_text_without_tools() {
        let parser = ToolParser::new();
        let text = r#"Hello! <tool name="test"/> Goodbye!"#;
        let result = parser.parse(text);

        let clean = result.text_without_tools();
        // Text segments are joined with proper spacing
        assert_eq!(clean, "Hello! Goodbye!");
    }

    #[test]
    fn test_parse_bracket_style_with_param() {
        let parser = ToolParser::new();
        let result = parser.parse("[start_focus_session:15]");

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
        assert_eq!(result.tool_calls[0].get_int("duration"), Some(15));
    }

    #[test]
    fn test_parse_bracket_style_without_param() {
        let parser = ToolParser::new();
        let result = parser.parse("[end_focus_session]");

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "end_focus_session");
    }

    #[test]
    fn test_parse_bracket_style_with_key_value() {
        let parser = ToolParser::new();
        let result = parser.parse("[start_focus_session:duration=45]");

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
        assert_eq!(result.tool_calls[0].get_int("duration"), Some(45));
    }

    #[test]
    fn test_parse_bracket_style_boolean() {
        let parser = ToolParser::new();
        let result = parser.parse("[end_focus_session:true]");

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "end_focus_session");
        assert_eq!(result.tool_calls[0].get_bool("completed"), Some(true));
    }

    #[test]
    fn test_parse_bracket_in_text() {
        let parser = ToolParser::new();
        let text = "Let me help you focus! [start_focus_session:25] Good luck!";
        let result = parser.parse(text);

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
        assert_eq!(result.tool_calls[0].get_int("duration"), Some(25));

        let text_only = result.text_without_tools();
        assert!(text_only.contains("help"));
        assert!(text_only.contains("Good luck"));
    }

    #[test]
    fn test_parse_multiple_bracket_formats() {
        let parser = ToolParser::new();
        let text = "Start session [start_focus_session:25] and check stats [get_session_stats]";
        let result = parser.parse(text);

        assert!(result.has_tools);
        assert_eq!(result.tool_calls.len(), 2);
        assert_eq!(result.tool_calls[0].name, "start_focus_session");
        assert_eq!(result.tool_calls[1].name, "get_session_stats");
    }
}
