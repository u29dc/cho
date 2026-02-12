//! Tool discovery command for agent self-discovery.
//!
//! `cho tools --json` returns a machine-readable catalog of all CLI capabilities.
//! `cho tools <name> --json` returns detail for a single tool.

use std::time::Instant;

use crate::envelope;
use crate::registry::{GLOBAL_FLAGS, TOOLS};

/// Runs the tools command.
///
/// Dispatched early (before client construction) since no auth is required.
pub fn run(name: Option<&str>, json_mode: bool, start: Instant) {
    if let Some(name) = name {
        run_detail(name, json_mode, start);
    } else {
        run_catalog(json_mode, start);
    }
}

/// Outputs the full tool catalog.
fn run_catalog(json_mode: bool, start: Instant) {
    if json_mode {
        let catalog = build_catalog_json();
        let output = envelope::emit_success("tools", catalog, start, Some(TOOLS.len()), None, None);
        println!("{output}");
    } else {
        print_catalog_text();
    }
}

/// Outputs detail for a single tool.
fn run_detail(name: &str, json_mode: bool, start: Instant) {
    let tool = TOOLS.iter().find(|t| t.name == name);

    match tool {
        Some(t) => {
            if json_mode {
                let detail = tool_to_json(t);
                let output = envelope::emit_success("tools", detail, start, None, None, None);
                println!("{output}");
            } else {
                print_tool_text(t);
            }
        }
        None => {
            if json_mode {
                let output = envelope::emit_error(
                    "tools",
                    "NOT_FOUND",
                    format!("Tool '{name}' not found"),
                    "Run 'cho tools --json' to list all available tools".to_string(),
                    start,
                );
                println!("{output}");
                std::process::exit(1);
            } else {
                eprintln!("Error: Tool '{name}' not found. Run 'cho tools' to list all.");
                std::process::exit(1);
            }
        }
    }
}

/// Builds the full catalog as a JSON value.
fn build_catalog_json() -> serde_json::Value {
    let tools: Vec<serde_json::Value> = TOOLS.iter().map(tool_to_json).collect();
    let flags: Vec<serde_json::Value> = GLOBAL_FLAGS
        .iter()
        .map(|f| {
            serde_json::json!({
                "name": f.name,
                "description": f.description,
                "default": f.default,
            })
        })
        .collect();

    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "tools": tools,
        "globalFlags": flags,
    })
}

/// Converts a single ToolMeta to JSON.
fn tool_to_json(t: &crate::registry::ToolMeta) -> serde_json::Value {
    let params: Vec<serde_json::Value> = t
        .parameters
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "type": p.param_type,
                "required": p.required,
                "description": p.description,
            })
        })
        .collect();

    serde_json::json!({
        "name": t.name,
        "command": t.command,
        "category": t.category,
        "description": t.description,
        "parameters": params,
        "outputFields": t.output_fields,
        "idempotent": t.idempotent,
        "rateLimitGroup": t.rate_limit,
        "example": t.example,
    })
}

/// Prints the catalog in human-readable format to stderr.
fn print_catalog_text() {
    let mut current_category = "";

    for tool in TOOLS.iter() {
        if tool.category != current_category {
            if !current_category.is_empty() {
                eprintln!();
            }
            eprintln!("  {}", tool.category.to_uppercase());
            current_category = tool.category;
        }
        eprintln!("    {:<30} {}", tool.name, tool.description);
    }

    eprintln!();
    eprintln!("  Use 'cho tools <name> --json' for tool detail.");
    eprintln!("  {} tools available.", TOOLS.len());
}

/// Prints a single tool in human-readable format.
fn print_tool_text(t: &crate::registry::ToolMeta) {
    eprintln!("  {}", t.name);
    eprintln!("  {}", t.description);
    eprintln!();
    eprintln!("  Command: {}", t.command);
    eprintln!("  Category: {}", t.category);
    eprintln!("  Idempotent: {}", t.idempotent);

    if let Some(rl) = t.rate_limit {
        eprintln!("  Rate limit: {rl}");
    }

    if !t.parameters.is_empty() {
        eprintln!();
        eprintln!("  Parameters:");
        for p in t.parameters {
            let req = if p.required { " (required)" } else { "" };
            eprintln!("    {:<20} {}{}", p.name, p.description, req);
        }
    }

    if !t.output_fields.is_empty() {
        eprintln!();
        eprintln!("  Output fields: {}", t.output_fields.join(", "));
    }

    eprintln!();
    eprintln!("  Example: {}", t.example);
}
