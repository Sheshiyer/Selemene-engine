use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const API_LIB_PATH: &str = "crates/noesis-api/src/lib.rs";
const AUTH_HANDLER_PATH: &str = "crates/noesis-api/src/handlers/auth.rs";
const USER_HANDLER_PATH: &str = "crates/noesis-api/src/handlers/users.rs";
const ADMIN_HANDLER_PATH: &str = "crates/noesis-api/src/handlers/admin.rs";
const INVENTORY_PATH: &str = "docs/baseline/api-route-inventory.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiRouteInventory {
    pub generated_at: String,
    pub inventory_scope: String,
    pub source: String,
    pub path_count: usize,
    pub route_count: usize,
    pub routes: Vec<RouteEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteEntry {
    pub path: String,
    pub auth_requirement: String,
    pub uses_orchestrator: bool,
    pub methods: Vec<RouteMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteMethod {
    pub method: String,
    pub handler: String,
}

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}

pub fn documented_route_inventory() -> ApiRouteInventory {
    let raw = fs::read_to_string(workspace_root().join(INVENTORY_PATH))
        .expect("documented route inventory should exist");
    serde_json::from_str(&raw).expect("documented route inventory should parse")
}

pub fn source_route_inventory() -> ApiRouteInventory {
    let api_source = fs::read_to_string(workspace_root().join(API_LIB_PATH))
        .expect("noesis-api source should be readable");
    let auth_source = fs::read_to_string(workspace_root().join(AUTH_HANDLER_PATH))
        .expect("auth handler source should be readable");
    let user_source = fs::read_to_string(workspace_root().join(USER_HANDLER_PATH))
        .expect("users handler source should be readable");
    let admin_source = fs::read_to_string(workspace_root().join(ADMIN_HANDLER_PATH))
        .expect("admin handler source should be readable");

    let auth_block = extract_block(
        &api_source,
        "let auth_routes = Router::new()",
        "let api_v1 = Router::new()",
    );
    let api_v1_block = extract_block(&api_source, "let api_v1 = Router::new()", "// Legacy");

    let mut routes = parse_route_block(
        &auth_block,
        "public",
        &api_source,
        &auth_source,
        &user_source,
        &admin_source,
    );
    routes.extend(parse_route_block(
        &api_v1_block,
        "bearer_or_api_key",
        &api_source,
        &auth_source,
        &user_source,
        &admin_source,
    ));

    let path_count = routes.len();
    let route_count = routes.iter().map(|route| route.methods.len()).sum();

    ApiRouteInventory {
        generated_at: "2026-03-13".to_string(),
        inventory_scope: "/api/v1".to_string(),
        source: API_LIB_PATH.to_string(),
        path_count,
        route_count,
        routes,
    }
}

fn extract_block(source: &str, start_marker: &str, end_marker: &str) -> String {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing marker: {}", start_marker));
    let tail = &source[start..];
    let end = tail
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing marker: {}", end_marker));
    tail[..end].to_string()
}

fn parse_route_block(
    block: &str,
    auth_requirement: &str,
    api_source: &str,
    auth_source: &str,
    user_source: &str,
    admin_source: &str,
) -> Vec<RouteEntry> {
    extract_route_calls(block)
        .into_iter()
        .map(|call| {
            let (path, handler_expr) = split_route_call(&call);
            let methods = parse_route_methods(&handler_expr);
            let uses_orchestrator = methods.iter().any(|method| {
                handler_uses_orchestrator(
                    &method.handler,
                    api_source,
                    auth_source,
                    user_source,
                    admin_source,
                )
            });

            RouteEntry {
                path,
                auth_requirement: auth_requirement.to_string(),
                uses_orchestrator,
                methods,
            }
        })
        .collect()
}

fn extract_route_calls(block: &str) -> Vec<String> {
    let mut calls = Vec::new();
    let mut search_from = 0;

    while let Some(relative_idx) = block[search_from..].find(".route(") {
        let route_start = search_from + relative_idx;
        let open_idx = route_start + ".route".len();
        let close_idx = find_matching_delimiter(block, open_idx, '(', ')');
        calls.push(block[open_idx + 1..close_idx].trim().to_string());
        search_from = close_idx + 1;
    }

    calls
}

fn split_route_call(call: &str) -> (String, String) {
    let first_quote = call.find('"').expect("route call should start with a path");
    let second_quote = call[first_quote + 1..]
        .find('"')
        .map(|idx| first_quote + 1 + idx)
        .expect("route path should terminate");
    let path = call[first_quote + 1..second_quote].to_string();
    let remainder = call[second_quote + 1..].trim_start();
    let handler_expr = remainder
        .strip_prefix(',')
        .expect("route path should be followed by a comma")
        .trim()
        .trim_end_matches(',')
        .trim()
        .to_string();
    (path, handler_expr)
}

fn parse_route_methods(handler_expr: &str) -> Vec<RouteMethod> {
    let method_specs = ["get", "post", "patch", "put", "delete"];
    let mut methods = Vec::new();
    let mut idx = 0;

    while idx < handler_expr.len() {
        let mut matched = false;

        for method in method_specs {
            let marker = format!("{}(", method);
            if handler_expr[idx..].starts_with(&marker) {
                let open_idx = idx + method.len();
                let close_idx = find_matching_delimiter(handler_expr, open_idx, '(', ')');
                let handler = handler_expr[open_idx + 1..close_idx].trim().to_string();
                methods.push(RouteMethod {
                    method: method.to_uppercase(),
                    handler,
                });
                idx = close_idx + 1;
                matched = true;
                break;
            }
        }

        if !matched {
            idx += 1;
        }
    }

    methods
}

fn handler_uses_orchestrator(
    handler: &str,
    api_source: &str,
    auth_source: &str,
    user_source: &str,
    admin_source: &str,
) -> bool {
    let handler_name = handler.rsplit("::").next().unwrap_or(handler);

    if matches!(
        handler_name,
        "status_handler"
            | "list_engines_handler"
            | "calculate_handler"
            | "validate_handler"
            | "engine_info_handler"
            | "list_workflows_handler"
            | "workflow_execute_handler"
            | "workflow_info_handler"
            | "system_health"
            | "system_workflows"
    ) {
        return true;
    }

    handler_body(handler_name, api_source)
        .or_else(|| handler_body(handler_name, auth_source))
        .or_else(|| handler_body(handler_name, user_source))
        .or_else(|| handler_body(handler_name, admin_source))
        .map(|body| body.contains("state.orchestrator"))
        .unwrap_or(false)
}

fn handler_body<'a>(handler_name: &str, source: &'a str) -> Option<&'a str> {
    let signature = format!("fn {}(", handler_name);
    let start = source.find(&signature)?;
    let after_signature = &source[start..];
    let open_idx = start + after_signature.find('{')?;
    let close_idx = find_matching_delimiter(source, open_idx, '{', '}');
    Some(&source[open_idx + 1..close_idx])
}

fn find_matching_delimiter(source: &str, open_idx: usize, open: char, close: char) -> usize {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut idx = open_idx;
    let mut in_string = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let ch = bytes[idx] as char;

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            idx += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            idx += 1;
            continue;
        }

        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return idx;
            }
        }

        idx += 1;
    }

    panic!("unmatched delimiter {} in source", open);
}
