use anyhow::Result;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AbridgeMcp {
    tool_router: ToolRouter<Self>,
}

impl AbridgeMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScreenshotParams {
    /// Whether to run OCR on the screenshot
    #[serde(default)]
    pub ocr: bool,
    /// Whether to include the view hierarchy
    #[serde(default)]
    pub hierarchy: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LogcatParams {
    /// Filter by app package name
    pub app: Option<String>,
    /// Filter by log tag
    pub tag: Option<String>,
    /// Minimum log level (verbose, debug, info, warn, error, fatal)
    #[serde(default = "default_level")]
    pub level: String,
    /// Number of recent lines
    #[serde(default = "default_lines")]
    pub lines: u32,
}

fn default_level() -> String {
    "verbose".to_string()
}

fn default_lines() -> u32 {
    50
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InputParams {
    /// Input type: "text", "tap", "swipe", "key", "clip"
    pub r#type: String,
    /// The value: text content, key name, or coordinates as "x,y" or "x1,y1,x2,y2"
    pub value: String,
    /// Duration for swipe in ms (optional)
    pub duration: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CrashParams {
    /// Path to source root for stacktrace mapping
    pub source_root: Option<String>,
}

#[tool_router]
impl AbridgeMcp {
    #[tool(description = "Capture a screenshot from the connected Android device. Returns base64 PNG, optional OCR text, and optional view hierarchy XML.")]
    async fn device_screenshot(
        &self,
        Parameters(params): Parameters<ScreenshotParams>,
    ) -> String {
        match crate::screen::capture(params.ocr, params.hierarchy) {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error capturing screenshot: {e}"),
        }
    }

    #[tool(description = "Get filtered logcat entries from the connected Android device. Can filter by app, tag, and log level.")]
    async fn device_logcat(
        &self,
        Parameters(params): Parameters<LogcatParams>,
    ) -> String {
        match crate::logcat::fetch(
            params.app.as_deref(),
            params.tag.as_deref(),
            &params.level,
            params.lines,
        ) {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error reading logcat: {e}"),
        }
    }

    #[tool(description = "Get current device state: focused activity, fragment backstack, display info, and memory stats.")]
    async fn device_state(&self) -> String {
        match crate::state::get_state(true) {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error getting device state: {e}"),
        }
    }

    #[tool(description = "Send input to the Android device. Types: 'text' (type text), 'tap' (value='x,y'), 'swipe' (value='x1,y1,x2,y2'), 'key' (value=home/back/enter/menu), 'clip' (set clipboard).")]
    async fn device_input(
        &self,
        Parameters(params): Parameters<InputParams>,
    ) -> String {
        let result = match params.r#type.as_str() {
            "text" => crate::input::input_text(&params.value),
            "tap" => {
                let coords: Vec<u32> = params
                    .value
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                if coords.len() == 2 {
                    crate::input::tap(coords[0], coords[1])
                } else {
                    return "Error: tap value must be 'x,y'".to_string();
                }
            }
            "swipe" => {
                let coords: Vec<u32> = params
                    .value
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                if coords.len() == 4 {
                    crate::input::swipe(
                        coords[0],
                        coords[1],
                        coords[2],
                        coords[3],
                        params.duration.unwrap_or(300),
                    )
                } else {
                    return "Error: swipe value must be 'x1,y1,x2,y2'".to_string();
                }
            }
            "key" => crate::input::key(&params.value),
            "clip" => crate::input::set_clipboard(&params.value),
            other => return format!("Unknown input type: {other}"),
        };

        match result {
            Ok(()) => format!("OK: {} '{}'", params.r#type, params.value),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get the most recent crash report: stacktrace, current activity, recent error logs, and a screenshot.")]
    async fn device_crash_report(
        &self,
        Parameters(_params): Parameters<CrashParams>,
    ) -> String {
        match crate::state::get_crash_report(true) {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| e.to_string()),
            Err(e) => format!("Error getting crash report: {e}"),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AbridgeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Android Bridge for AI-Assisted Development. Provides device screenshot/OCR, logcat, input control, state inspection, and crash reports.")
    }
}

/// Start the MCP server on stdio.
pub async fn serve() -> Result<()> {
    tracing::info!("Starting abridge MCP server on stdio");

    let service = AbridgeMcp::new();
    let server = service
        .serve(rmcp::transport::stdio())
        .await?;
    server.waiting().await?;

    Ok(())
}
