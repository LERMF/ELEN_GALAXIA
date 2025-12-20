//! # AGENT_CGI (GOD_MODE) Orchestrator
//!
//! The central durable execution engine for NEXUS_OMNI.
//! Implements ReAct-based autonomous task orchestration with Zero Trust authentication.
//!
//! ## Architecture
//! - Zero Trust JWT validation via CF-Access
//! - ReAct loop with Workers AI inference
//! - Service binding RPC for tool dispatch
//! - D1/Vectorize for state persistence

use worker::*;

mod auth;
mod react;
mod rpc;
mod state;
mod tools;

use auth::validate_cf_access_jwt;
use react::ReActLoop;
use state::AgentState;

/// Main fetch handler - the entry point for all requests
#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    // Initialize router
    Router::new()
        .get("/", |_, _| Response::ok("🧠 AGENT_CGI (GOD_MODE) v0.1.0 - NEXUS_OMNI Orchestrator"))
        .get("/health", |_, _| {
            Response::from_json(&serde_json::json!({
                "status": "operational",
                "module": "agent-cgi",
                "version": "0.1.0"
            }))
        })
        .post_async("/agent/invoke", handle_invoke)
        .run(req, env)
        .await
}

/// 🎯 Main agent invocation handler
/// Validates JWT, loads context, runs ReAct loop
async fn handle_invoke(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // ═══════════════════════════════════════════════════════════════════════
    // 🛡️ ZERO TRUST GATE: Validate CF-Access-JWT
    // ═══════════════════════════════════════════════════════════════════════
    let jwt = req
        .headers()
        .get("Cf-Access-Jwt-Assertion")?
        .ok_or_else(|| Error::from("Missing Cf-Access-Jwt-Assertion header"))?;

    let admin_email = ctx.env.var("ADMIN_EMAIL")?.to_string();
    let team_domain = ctx.env.var("CF_TEAM_DOMAIN")?.to_string();

    let claims = validate_cf_access_jwt(&jwt, &team_domain, &admin_email).await?;
    console_log!("✅ Authenticated admin: {}", claims.email);

    // ═══════════════════════════════════════════════════════════════════════
    // 📥 PARSE REQUEST
    // ═══════════════════════════════════════════════════════════════════════
    let body: serde_json::Value = req.json().await?;
    let goal = body
        .get("goal")
        .and_then(|g| g.as_str())
        .ok_or_else(|| Error::from("Missing 'goal' in request body"))?;

    console_log!("🎯 Goal received: {}", goal);

    // ═══════════════════════════════════════════════════════════════════════
    // 📦 LOAD CONTEXT
    // ═══════════════════════════════════════════════════════════════════════
    let state = AgentState::new(goal);
    console_log!("📦 Session initialized: {}", state.session_id);

    // ═══════════════════════════════════════════════════════════════════════
    // 🧠 RUN REACT LOOP
    // ═══════════════════════════════════════════════════════════════════════
    let max_iterations: u32 = ctx
        .env
        .var("MAX_REACT_ITERATIONS")
        .ok()
        .and_then(|v| v.to_string().parse().ok())
        .unwrap_or(5);

    let result = ReActLoop::new(&ctx.env)
        .with_state(state)
        .with_max_iterations(max_iterations)
        .execute(goal)
        .await?;

    console_log!(
        "✅ ReAct complete: {} iterations, final_answer: {}",
        result.iterations,
        result.final_answer.is_some()
    );

    // ═══════════════════════════════════════════════════════════════════════
    // 📤 RETURN RESPONSE
    // ═══════════════════════════════════════════════════════════════════════
    Response::from_json(&result)
}
