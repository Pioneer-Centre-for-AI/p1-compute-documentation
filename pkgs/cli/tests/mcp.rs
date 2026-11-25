mod common;

use std::io::Write;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

use aicentre::context::{Ctx, Output};
use aicentre::http::Fetched;
use aicentre::mcp::server::{mcp, serve, serve_with};
use aicentre::mcp::tools::Fetcher;
use common::{Sandbox, dtu_host};
use serde_json::{Value, json};
use tokio::io::{AsyncRead, BufReader, ReadBuf};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

struct Input {
    rx: UnboundedReceiver<Vec<u8>>,
    chunk: Vec<u8>,
    at: usize,
}

impl AsyncRead for Input {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = &mut *self;
        if this.at >= this.chunk.len() {
            match this.rx.poll_recv(cx) {
                Poll::Ready(Some(chunk)) => {
                    this.chunk = chunk;
                    this.at = 0;
                }
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => return Poll::Pending,
            }
        }
        let n = buf.remaining().min(this.chunk.len() - this.at);
        buf.put_slice(&this.chunk[this.at..this.at + n]);
        this.at += n;
        Poll::Ready(Ok(()))
    }
}

#[derive(Default)]
struct Seen {
    lines: Vec<Value>,
    raw: Vec<String>,
}

/// The agent tool's end.
#[derive(Clone)]
struct Client {
    input: Arc<Mutex<Option<UnboundedSender<Vec<u8>>>>>,
    seen: Arc<Mutex<Seen>>,
    next: Arc<Mutex<u64>>,
    partial: Arc<Mutex<String>>,
}

impl Write for Client {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut partial = self.partial.lock().unwrap();
        partial.push_str(&String::from_utf8_lossy(buf));
        while let Some(end) = partial.find('\n') {
            let line: String = partial.drain(..=end).collect();
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
            let m: Value = serde_json::from_str(line).unwrap();
            let mut seen = self.seen.lock().unwrap();
            seen.raw.push(line.to_string());
            seen.lines.push(m);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Client {
    fn new() -> (Client, BufReader<Input>) {
        let (tx, rx) = unbounded_channel();
        let client = Client {
            input: Arc::new(Mutex::new(Some(tx))),
            seen: Arc::default(),
            next: Arc::default(),
            partial: Arc::default(),
        };
        (client, BufReader::new(Input { rx, chunk: vec![], at: 0 }))
    }

    fn ctx(&self, sb: &Sandbox) -> Ctx {
        let out = Output::new(Box::new(self.clone()), false, Some(100));
        sb.ctx_with(|c| c.out = out)
    }

    fn write_line(&self, line: &str) {
        if let Some(tx) = self.input.lock().unwrap().as_ref() {
            let _ = tx.send(format!("{line}\n").into_bytes());
        }
    }

    fn send(&self, m: &Value) {
        self.write_line(&m.to_string());
    }

    fn post(&self, method: &str, params: Value, id: Option<Value>) {
        let mut m = json!({ "jsonrpc": "2.0" });
        if let Some(id) = id {
            m["id"] = id;
        }
        m["method"] = json!(method);
        m["params"] = params;
        self.send(&m);
    }

    async fn reply(&self, id: &Value) -> Value {
        loop {
            if let Some(m) = self.seen.lock().unwrap().lines.iter().find(|l| l["id"] == *id) {
                return m.clone();
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    async fn call(&self, method: &str, params: Value) -> Value {
        let id = {
            let mut next = self.next.lock().unwrap();
            *next += 1;
            json!(*next)
        };
        self.post(method, params, Some(id.clone()));
        self.reply(&id).await
    }

    async fn tool(&self, name: &str, args: Value) -> String {
        let r = self.call("tools/call", json!({ "name": name, "arguments": args })).await;
        r["result"]["content"][0]["text"].as_str().unwrap().to_string()
    }

    async fn start(&self) -> Value {
        self.call(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0" }
            }),
        )
        .await
    }

    fn lines(&self) -> Vec<Value> {
        self.seen.lock().unwrap().lines.clone()
    }

    fn close(&self) {
        self.input.lock().unwrap().take();
    }
}

fn sandbox() -> Sandbox {
    let sb = Sandbox::new();
    sb.file(".ssh/config", "");
    dtu_host(&sb);
    sb
}

#[tokio::test]
async fn speaks_only_json_rpc_on_stdout_and_lists_the_tools() {
    let sb = sandbox();
    sb.answer("bjobs", "123|RUN|p1|train|1*n-62-30-1|60 second(s)|0:59 L|-\n");
    let (c, input) = Client::new();
    let ctx = c.ctx(&sb);
    let (code, ()) = tokio::join!(serve(&ctx, input), async {
        let init = c.start().await;
        assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(init["result"]["capabilities"], json!({ "tools": {} }));
        assert_eq!(init["result"]["serverInfo"]["name"], "aicentre");
        assert_eq!(init["result"]["serverInfo"]["version"], "0.1.0");
        let listed = c.call("tools/list", json!({})).await;
        let names: Vec<&str> = listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["p1_docs", "p1_jobs", "p1_check"]);
        let jobs: Value = serde_json::from_str(&c.tool("p1_jobs", json!({})).await).unwrap();
        assert_eq!(jobs["jobs"][0]["provider"], "dtu");
        assert_eq!(jobs["jobs"][0]["id"], "123");
        assert_eq!(jobs["jobs"][0]["state"], "running");
        let bad = c
            .call("tools/call", json!({ "name": "p1_check", "arguments": { "connect": "yes" } }))
            .await;
        assert_eq!(bad["result"]["isError"], true);
        let unknown = c.call("tools/call", json!({ "name": "p1_shell", "arguments": {} })).await;
        assert_eq!(unknown["error"]["code"], -32602);
        assert_eq!(c.call("resources/list", json!({})).await["error"]["code"], -32601);
        c.close();
    });
    assert_eq!(code, 0);
    assert_eq!(sb.err.text(), "");
}

#[tokio::test]
async fn a_cancelled_call_gets_no_reply_and_the_others_still_do() {
    let sb = sandbox();
    sb.answer("bjobs", "123|RUN|p1|train|1*n-62-30-1|60 second(s)|0:59 L|-\n");
    let (c, input) = Client::new();
    let ctx = c.ctx(&sb);
    c.post("initialize", json!({ "capabilities": {} }), Some(json!(1)));
    c.post("tools/call", json!({ "name": "p1_jobs", "arguments": {} }), Some(json!("call-1")));
    c.post("notifications/cancelled", json!({ "requestId": "call-1" }), None);
    c.post("tools/call", json!({ "name": "p1_jobs", "arguments": {} }), Some(json!("call-2")));
    c.close();
    assert_eq!(serve(&ctx, input).await, 0);
    let lines = c.lines();
    assert!(!lines.iter().any(|l| l["id"] == "call-1"));
    assert!(lines.iter().any(|l| l["id"] == "call-2" && l["result"]["isError"] == false));
}

#[tokio::test]
async fn an_agent_that_asks_for_the_jobs_again_and_again_reaches_the_cluster_once() {
    let sb = sandbox();
    sb.answer("bjobs", "123|RUN|p1|train|1*n-62-30-1|60 second(s)|0:59 L|-\n");
    let (c, input) = Client::new();
    let ctx = c.ctx(&sb);
    c.post("initialize", json!({ "capabilities": {} }), Some(json!(1)));
    for id in 2..7 {
        c.post("tools/call", json!({ "name": "p1_jobs", "arguments": {} }), Some(json!(id)));
    }
    c.close();
    assert_eq!(serve(&ctx, input).await, 0);
    let lines = c.lines();
    let answers: Vec<&Value> = lines.iter().filter(|l| l["id"].as_i64() > Some(1)).collect();
    assert_eq!(answers.len(), 5);
    assert!(answers.iter().all(|a| a["result"] == answers[0]["result"]));
    assert_eq!(sb.read("calls").matches("bjobs").count(), 1, "{}", sb.read("calls"));
}

#[tokio::test]
async fn the_same_question_in_other_words_does_not_reach_the_cluster_again() {
    let sb = sandbox();
    sb.answer("bjobs", "123|RUN|p1|train|1*n-62-30-1|60 second(s)|0:59 L|-\n");
    let (c, input) = Client::new();
    let ctx = c.ctx(&sb);
    c.post("initialize", json!({ "capabilities": {} }), Some(json!(1)));
    for (id, arguments) in [
        json!({ "providers": ["dtu"] }),
        json!({ "providers": ["dtu", "dtu"] }),
        json!({ "providers": ["dtu"], "connect": true }),
    ]
    .into_iter()
    .enumerate()
    {
        let call = json!({ "name": "p1_jobs", "arguments": arguments });
        c.post("tools/call", call, Some(json!(id + 2)));
    }
    c.close();
    assert_eq!(serve(&ctx, input).await, 0);
    assert_eq!(sb.read("calls").matches("bjobs").count(), 1, "{}", sb.read("calls"));
}

#[tokio::test]
async fn an_answer_with_a_failure_in_it_is_not_asked_for_again_at_once_either() {
    let sb = sandbox();
    sb.answer("bjobs-stderr", "bjobs: LSF is down\n");
    sb.answer("bjobs.code", "1");
    let (c, input) = Client::new();
    let ctx = c.ctx(&sb);
    c.post("initialize", json!({ "capabilities": {} }), Some(json!(1)));
    for id in 2..5 {
        c.post("tools/call", json!({ "name": "p1_jobs", "arguments": {} }), Some(json!(id)));
    }
    c.close();
    assert_eq!(serve(&ctx, input).await, 0);
    let lines = c.lines();
    let answers: Vec<&Value> = lines.iter().filter(|l| l["id"].as_i64() > Some(1)).collect();
    assert_eq!(answers.len(), 3);
    assert!(answers[0]["result"]["content"][0]["text"].as_str().unwrap().contains("LSF is down"));
    assert_eq!(sb.read("calls").matches("bjobs").count(), 1, "{}", sb.read("calls"));
}

#[tokio::test]
async fn refuses_a_terminal_an_agent_tool_starts_it() {
    let sb = sandbox();
    assert_eq!(mcp(&sb.ctx_with(|c| c.interactive = true)).await.unwrap(), 2);
    assert!(sb.err.text().contains("aicentre mcp --help"));
}

#[tokio::test]
async fn p1_docs_reads_a_page_through_the_fetcher() {
    let sb = sandbox();
    let fetched: Arc<Mutex<Vec<String>>> = Arc::default();
    let log = fetched.clone();
    let fetcher: Fetcher = Arc::new(move |url: String| {
        log.lock().unwrap().push(url.clone());
        Box::pin(async move {
            let status = if url.ends_with("missing.md") { 404 } else { 200 };
            Ok(Fetched { status, body: format!("# {url}").into_bytes() })
        })
    });
    let (c, input) = Client::new();
    let ctx = c.ctx(&sb);
    tokio::join!(serve_with(&ctx, input, fetcher), async {
        c.start().await;
        assert_eq!(
            c.tool("p1_docs", json!({ "path": "providers/dtu" })).await,
            "Source: https://hpc.aicentre.dk/providers/dtu.md\n\n# https://hpc.aicentre.dk/providers/dtu.md"
        );
        assert_eq!(
            c.tool("p1_docs", json!({ "path": "missing" })).await,
            "Could not read the docs: https://hpc.aicentre.dk/missing.md: HTTP 404"
        );
        assert!(
            c.tool("p1_docs", json!({}))
                .await
                .starts_with("Source: https://hpc.aicentre.dk/llms.txt\n")
        );
        c.close();
    });
    assert_eq!(fetched.lock().unwrap().len(), 3);
}
