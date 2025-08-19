use koto_ls::info_cache::InfoCache;
use koto_ls::source_info::{Location, SourceInfo};
use std::{str::FromStr, sync::Arc};
use tower_lsp_server::lsp_types::{Position, Range, Uri};

fn test_uri() -> Arc<Uri> {
    Arc::new(Uri::from_str("file:///test.koto").unwrap())
}

fn position(line: u32, character: u32) -> Position {
    Position { line, character }
}

fn range_at_position(line: u32, character: u32) -> Range {
    Range::new(position(line, character), position(line, character))
}

fn location_at_position(uri: Arc<Uri>, line: u32, character: u32) -> Location {
    Location {
        uri,
        range: range_at_position(line, character),
    }
}

#[test]
fn test_completion_simple_variables() {
    let script = "\
x = 42
y = \"hello\"
z = true
x + 
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 3, 3); // After "x + "
    let completions = info.get_available_definitions_at_location(location);

    assert_eq!(completions.len(), 3);

    let names: Vec<String> = completions
        .iter()
        .map(|d| d.id.as_str().to_string())
        .collect();
    assert!(names.contains(&"x".to_string()));
    assert!(names.contains(&"y".to_string()));
    assert!(names.contains(&"z".to_string()));
}

#[test]
fn test_completion_function_scope() {
    let script = "\
x = 1
foo = |a, b|
  local_var = a + b
  x + local_var + 
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 3, 16); // Before the end of the function
    let completions = info.get_available_definitions_at_location(location);

    let names: Vec<String> = completions
        .iter()
        .map(|d| d.id.as_str().to_string())
        .collect();
    assert!(names.contains(&"x".to_string())); // global
    assert!(names.contains(&"a".to_string())); // parameter
    assert!(names.contains(&"b".to_string())); // parameter
    assert!(names.contains(&"local_var".to_string())); // local variable
}

#[test]
fn test_completion_nested_scopes() {
    let script = "\
outer = 1
f = |x|
  inner = 2
  g = |y|
    nested = 3
    outer + inner + x + y + 
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 5, 26); // Before the end of the inner function
    let completions = info.get_available_definitions_at_location(location);

    let names: Vec<String> = completions
        .iter()
        .map(|d| d.id.as_str().to_string())
        .collect();
    assert!(names.contains(&"outer".to_string())); // global
    assert!(names.contains(&"x".to_string())); // outer function parameter
    assert!(names.contains(&"inner".to_string())); // outer function local
    assert!(names.contains(&"y".to_string())); // inner function parameter
    assert!(names.contains(&"nested".to_string())); // current scope
}

#[test]
fn test_completion_excludes_out_of_scope() {
    let script = "\
global_var = 1
f = |param|
  local_var = 2
  local_var + param
g = |other_param|
  other_local = 3
  global_var + other_local
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    // Look for a position inside the second function where other_local would be defined
    let location = location_at_position(test_uri(), 6, 15); // After "other_local"
    let completions = info.get_available_definitions_at_location(location);

    let names: Vec<String> = completions
        .iter()
        .map(|d| d.id.as_str().to_string())
        .collect();

    // Should have access to globals and current function scope
    assert!(names.contains(&"global_var".to_string()));

    // Should NOT have access to:
    assert!(!names.contains(&"param".to_string())); // from different function
    assert!(!names.contains(&"local_var".to_string())); // from different function
}

#[test]
fn test_completion_top_level() {
    let script = "\
x = 1
y = 2
f = |a| a * 2
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 3, 0); // Start of line 4
    let completions = info.get_available_definitions_at_location(location);

    let names: Vec<String> = completions
        .iter()
        .map(|d| d.id.as_str().to_string())
        .collect();
    assert!(names.contains(&"x".to_string()));
    assert!(names.contains(&"y".to_string()));
    assert!(names.contains(&"f".to_string()));
}

#[test]
fn test_completion_empty_script() {
    let script = "";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 0, 0);
    let completions = info.get_available_definitions_at_location(location);

    assert_eq!(completions.len(), 0);
}

#[test]
fn test_debug_simple_completion() {
    let script = "\
x = 42
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 0, 6); // After "x = 42"
    let completions = info.get_available_definitions_at_location(location);

    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].id.as_str(), "x");
}

#[test]
fn test_completion_variable_shadowing() {
    let script = "\
x = \"global\"
f = |x|
  x = x.upper()
  x + 
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 3, 5); // After "x + "
    let completions = info.get_available_definitions_at_location(location);

    let names: Vec<String> = completions
        .iter()
        .map(|d| d.id.as_str().to_string())
        .collect();

    // Should contain x (the redefined local one)
    assert!(names.contains(&"x".to_string()));

    // Verify we get the local definitions
    let x_definitions: Vec<_> = completions
        .iter()
        .filter(|d| d.id.as_str() == "x")
        .collect();
    assert!(!x_definitions.is_empty());
}

#[test]
fn test_completion_with_imports() {
    let script = "\
import foo
from bar import baz
local_var = 1
foo + baz +
";

    let mut info_cache = InfoCache::default();
    let info = SourceInfo::new(script.to_string(), test_uri(), &mut info_cache);

    let location = location_at_position(test_uri(), 3, 11); // After "foo + baz + "
    let completions = info.get_available_definitions_at_location(location);

    let names: Vec<String> = completions
        .iter()
        .map(|d| d.id.as_str().to_string())
        .collect();
    assert!(names.contains(&"foo".to_string()));
    assert!(names.contains(&"baz".to_string()));
    assert!(names.contains(&"local_var".to_string()));
}