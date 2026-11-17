use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use serde_json::{Map, Value, json};
use tokio::io::{AsyncBufRead, AsyncBufReadExt};
use tokio::task::{JoinSet, LocalSet};

use crate::context::Ctx;
use crate::mcp::tools::{Fetcher, MCP_INSTRUCTIONS, P1Tool, P1Tools, web_fetcher};

pub const PROTOCOLS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const NO_METHOD: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// A request id that a lookup can match: a string or a number.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Str(String),
    Num(u64),
}

fn key(id: &Value) -> Option<Key> {
    match id {
        Value::String(s) => Some(Key::Str(s.clone())),
        Value::Number(n) => n.as_f64().map(|f| Key::Num((f + 0.0).to_bits())),
        _ => None,
    }
}

fn shown(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => "nothing".into(),
    }
}

/// `value.name`
fn prop<'a>(v: Option<&'a Value>, name: &str) -> Option<&'a Value> {
    v.and_then(Value::as_object).and_then(|o| o.get(name))
}

/// Null counts as missing, as with `??`.
fn present(v: Option<&Value>) -> Option<&Value> {
    v.filter(|v| !v.is_null())
}

fn pointer(path: &str, part: &str) -> String {
    format!("{path}/{}", part.replace('~', "~0").replace('/', "~1"))
}

/// Typebox's errors, in its order.
fn errors(schema: &Value, value: &Value, path: &str, out: &mut Vec<(String, String)>) {
    let at = |message: String| (path.to_string(), message);
    if let Some(kind) = schema.get("type").and_then(Value::as_str) {
        let fits = match kind {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "boolean" => value.is_boolean(),
            "integer" => value.as_f64().is_some_and(|f| f.is_finite() && f.fract() == 0.0),
            "number" => value.is_number(),
            "null" => value.is_null(),
            _ => true,
        };
        if !fits {
            out.push(at(format!("must be {kind}")));
        }
    }
    if let Some(object) = value.as_object()
        && let Some(properties) = schema.get("properties").and_then(Value::as_object)
    {
        for (name, sub) in properties {
            if let Some(v) = object.get(name) {
                errors(sub, v, &pointer(path, name), out);
            }
        }
    }
    if let Some(items) = value.as_array()
        && let Some(sub) = schema.get("items")
    {
        for (i, v) in items.iter().enumerate() {
            errors(sub, v, &pointer(path, &i.to_string()), out);
        }
    }
    if let Some(constant) = schema.get("const")
        && constant != value
    {
        out.push(at("must be equal to constant".into()));
    }
    if let Some(branches) = schema.get("anyOf").and_then(Value::as_array) {
        let mut failed = Vec::new();
        let mut matched = false;
        for branch in branches {
            let mut found = Vec::new();
            errors(branch, value, path, &mut found);
            matched |= found.is_empty();
            failed.extend(found);
        }
        if !matched {
            out.extend(failed);
            out.push(at("must match a schema in anyOf".into()));
        }
    }
}

fn invalid(t: &P1Tool, args: &Value) -> Option<String> {
    let mut found = Vec::new();
    errors(&t.parameters, args, "", &mut found);
    let (path, message) = found.into_iter().next()?;
    Some(format!("{} {message}", if path.is_empty() { "arguments" } else { &path }))
}

fn listing(t: &P1Tool) -> Value {
    json!({
        "name": t.name,
        "title": t.label,
        "description": t.description,
        "inputSchema": t.parameters,
        "annotations": {
            "title": t.label,
            "readOnlyHint": true,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": true
        }
    })
}

#[derive(Default)]
struct State {
    calls: HashSet<Key>,
    dropped: HashSet<Key>,
}

struct Server {
    ctx: Ctx,
    tools: P1Tools,
    state: RefCell<State>,
}

impl Server {
    fn send(&self, fields: Value) {
        let mut m = Map::new();
        m.insert("jsonrpc".into(), "2.0".into());
        if let Value::Object(rest) = fields {
            m.extend(rest);
        }
        self.ctx.out.write(&format!("{}\n", Value::Object(m)));
    }

    fn fail(&self, id: Value, code: i64, message: &str) {
        self.send(json!({ "id": id, "error": { "code": code, "message": message } }));
    }

    fn cancelled(&self, call: &Key) {
        let mut st = self.state.borrow_mut();
        if st.calls.contains(call) {
            st.dropped.insert(call.clone());
        }
    }

    async fn call(&self, params: Option<&Value>) -> Value {
        let name = prop(params, "name");
        let Some(t) = name.and_then(Value::as_str).and_then(|n| self.tools.get(n)) else {
            return json!({ "error": format!("Unknown tool: {}", shown(name)) });
        };
        let empty = json!({});
        let args = present(prop(params, "arguments")).unwrap_or(&empty);
        if let Some(wrong) = invalid(t, args) {
            return json!({ "content": [{ "type": "text", "text": wrong }], "isError": true });
        }
        match self.tools.run(t.name, args).await {
            Ok(said) => {
                let text = if said.text.is_empty() { "(no output)".into() } else { said.text };
                json!({ "content": [{ "type": "text", "text": text }], "isError": false })
            }
            Err(e) => {
                json!({ "content": [{ "type": "text", "text": e.to_string() }], "isError": true })
            }
        }
    }

    async fn tools_call(self: Rc<Self>, id: Value, params: Option<Value>) {
        let call = key(&id);
        let result = self.call(params.as_ref()).await;
        if let Some(call) = &call {
            let mut st = self.state.borrow_mut();
            st.calls.remove(call);
            if st.dropped.remove(call) {
                return;
            }
        }
        match result.get("error") {
            Some(error) => self.fail(id, INVALID_PARAMS, error.as_str().unwrap_or_default()),
            None => self.send(json!({ "id": id, "result": result })),
        }
    }

    /// Only tools/call runs as a task.
    fn handle(self: &Rc<Self>, m: &Value, running: &mut JoinSet<()>) {
        let m = Some(m);
        let id = present(prop(m, "id"));
        let Some(method) = prop(m, "method") else { return };
        if method == "notifications/cancelled" {
            if let Some(k) = prop(prop(m, "params"), "requestId").and_then(key) {
                self.cancelled(&k);
            }
            return;
        }
        let Some(id) = id.cloned() else { return };
        let params = present(prop(m, "params"));
        match method.as_str().unwrap_or_default() {
            "initialize" => {
                let requested = prop(params, "protocolVersion").and_then(Value::as_str);
                let protocol = if let Some(requested) = requested.filter(|r| PROTOCOLS.contains(r))
                {
                    requested
                } else {
                    PROTOCOLS[0]
                };
                self.send(json!({
                    "id": id,
                    "result": {
                        "protocolVersion": protocol,
                        "capabilities": { "tools": {} },
                        "serverInfo": {
                            "name": "aicentre",
                            "title": "aicentre: P1 compute",
                            "version": self.ctx.version
                        },
                        "instructions": MCP_INSTRUCTIONS
                    }
                }));
            }
            "ping" => self.send(json!({ "id": id, "result": {} })),
            "tools/list" => {
                let tools: Vec<Value> = self.tools.tools.iter().map(listing).collect();
                self.send(json!({ "id": id, "result": { "tools": tools } }));
            }
            "tools/call" => {
                if let Some(k) = key(&id) {
                    self.state.borrow_mut().calls.insert(k);
                }
                running.spawn_local(Rc::clone(self).tools_call(id, params.cloned()));
            }
            _ => self.fail(id, NO_METHOD, &format!("Method not found: {}", shown(Some(method)))),
        }
    }

    fn receive(self: &Rc<Self>, line: &str, running: &mut JoinSet<()>) {
        if line.trim().is_empty() {
            return;
        }
        let Ok(parsed) = serde_json::from_str::<Value>(line) else {
            self.fail(Value::Null, PARSE_ERROR, "Parse error");
            return;
        };
        let messages = match parsed {
            Value::Array(items) => items,
            one => vec![one],
        };
        for m in &messages {
            if m.is_object() || m.is_array() {
                self.handle(m, running);
            } else {
                self.fail(Value::Null, INVALID_REQUEST, "Invalid request");
            }
        }
    }
}

pub async fn mcp(ctx: &Ctx) -> anyhow::Result<i32> {
    Ok(serve(ctx, tokio::io::BufReader::new(tokio::io::stdin())).await)
}

pub async fn serve<R: AsyncBufRead + Unpin + Send>(ctx: &Ctx, input: R) -> i32 {
    serve_with(ctx, input, web_fetcher()).await
}

pub async fn serve_with<R: AsyncBufRead + Unpin + Send>(
    ctx: &Ctx,
    mut input: R,
    fetcher: Fetcher,
) -> i32 {
    if ctx.interactive {
        ctx.err.write(
            "aicentre: mcp is started by an agent tool, not typed in a terminal. aicentre mcp --help shows how to add it.\n",
        );
        return 2;
    }
    let server = Rc::new(Server {
        ctx: ctx.clone(),
        tools: P1Tools::new(ctx, fetcher),
        state: RefCell::new(State::default()),
    });
    LocalSet::new()
        .run_until(async move {
            let mut running = JoinSet::new();
            let mut buf = Vec::new();
            loop {
                buf.clear();
                match input.read_until(b'\n', &mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
                let text = String::from_utf8_lossy(&buf);
                let text = text.strip_suffix('\n').unwrap_or(&text);
                let text = text.strip_suffix('\r').unwrap_or(text);
                for line in text.split('\r') {
                    server.receive(line, &mut running);
                }
                // New calls start before the next line.
                tokio::task::yield_now().await;
                while running.try_join_next().is_some() {}
            }
            while running.join_next().await.is_some() {}
            0
        })
        .await
}
