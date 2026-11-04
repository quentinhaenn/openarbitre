//! Spike S5: stdio MCP server whose tools come from a runtime catalogue (rmcp, `local` feature).

use std::cell::RefCell;
use std::rc::Rc;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value, json};

/// Stand-in for `arbitre-core::Store`: `&self`, interior mutability, futures not `Send`.
#[derive(Default)]
struct FakeStore {
    notes: RefCell<Vec<String>>,
}

impl FakeStore {
    async fn add(self: Rc<Self>, text: String) -> Result<usize, String> {
        let me = self.clone(); // Rc held across an await: the future is !Send.
        tokio::task::yield_now().await;
        if text.trim().is_empty() {
            return Err("empty note".into());
        }
        me.notes.borrow_mut().push(text);
        Ok(me.notes.borrow().len())
    }

    async fn list(&self) -> Vec<String> {
        tokio::task::yield_now().await;
        self.notes.borrow().clone()
    }
}

/// One catalogue entry, as ST-03 would declare it (name, description, input schema).
struct Action {
    name: &'static str,
    description: &'static str,
    schema: Value,
    module: &'static str,
}

fn catalogue(enabled: &[&str]) -> Vec<Action> {
    let all = vec![
        Action {
            name: "notes.add",
            description: "Add a note.",
            schema: json!({"type":"object","properties":{"text":{"type":"string"}},"required":["text"]}),
            module: "notes",
        },
        Action {
            name: "notes.list",
            description: "List notes.",
            schema: json!({"type":"object","properties":{}}),
            module: "notes",
        },
        Action {
            name: "plan.today",
            description: "Today's plan.",
            schema: json!({"type":"object","properties":{}}),
            module: "plan",
        },
    ];
    all.into_iter().filter(|a| enabled.contains(&a.module)).collect()
}

struct Server {
    actions: Vec<Action>,
    store: Rc<FakeStore>,
}

fn tool_of(a: &Action) -> Tool {
    let Value::Object(schema) = a.schema.clone() else { unreachable!() };
    Tool::new(a.name, a.description, schema)
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.actions.iter().map(tool_of).collect()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.actions.iter().find(|a| a.name == name).map(tool_of)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if !self.actions.iter().any(|a| a.name == request.name) {
            // Unroutable: protocol error. ST-07 would say "module not enabled" (CONFLICT-6).
            return Err(ErrorData::invalid_params(
                format!("unknown or disabled action: {}", request.name),
                Some(json!({"code": "module_not_enabled"})),
            ));
        }
        let args = request.arguments.unwrap_or_else(Map::new);
        let result = match &*request.name {
            "notes.add" => {
                let text = args.get("text").and_then(Value::as_str).unwrap_or("").to_owned();
                match self.store.clone().add(text).await {
                    Ok(n) => CallToolResult::structured(json!({"count": n})),
                    Err(e) => CallToolResult::structured_error(json!({"code": "invalid", "message": e})),
                }
            }
            "notes.list" => CallToolResult::success(
                self.store.list().await.into_iter().map(ContentBlock::text).collect(),
            ),
            other => CallToolResult::error(vec![ContentBlock::text(format!("{other}: not wired"))]),
        };
        Ok(result.into())
    }
}

fn main() {
    let enabled: Vec<String> = std::env::var("S5_MODULES")
        .unwrap_or_else(|_| "notes".into())
        .split(',')
        .map(str::to_owned)
        .collect();
    let enabled: Vec<&str> = enabled.iter().map(String::as_str).collect();
    let server = Server { actions: catalogue(&enabled), store: Rc::default() };

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, async move {
        let running = server.serve(rmcp::transport::stdio()).await.expect("initialize");
        running.waiting().await.expect("service loop");
    });
}
