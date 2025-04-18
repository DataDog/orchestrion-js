use std::path::PathBuf;

use nodejs_semver::{Range, Version};

use crate::function_query::FunctionQuery;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub enum InstrumentationOperator {
    #[serde(rename = "traceCallback")]
    Callback,
    #[serde(rename = "tracePromise")]
    Promise,
    #[serde(rename = "traceSync")]
    Sync,
    #[serde(rename = "traceAsync")]
    Async,
}

impl InstrumentationOperator {
    pub fn as_str(&self) -> &str {
        match self {
            InstrumentationOperator::Callback => "traceCallback",
            InstrumentationOperator::Promise => "tracePromise",
            InstrumentationOperator::Sync => "traceSync",
            InstrumentationOperator::Async => "traceAsync",
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct InstrumentationConfig {
    pub module_name: String,
    pub version_range: Range,
    pub file_path: PathBuf,
    pub function_query: FunctionQuery,
    pub operator: InstrumentationOperator,
    pub channel_name: String,
}

#[derive(Deserialize, Clone)]
pub struct OrchestrionConfig {
    pub instrumentations: Vec<InstrumentationConfig>,
    #[serde(default = "OrchestrionConfig::dc_module_default")]
    pub dc_module: String,
}

impl OrchestrionConfig {
    fn dc_module_default() -> String {
        "diagnostics_channel".to_string()
    }
}

impl InstrumentationConfig {
    pub fn matches(&self, module_name: &str, version: &str, file_path: &PathBuf) -> bool {
        let version: Version = match version.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        self.module_name == module_name
            && version.satisfies(&self.version_range)
            && self.file_path == *file_path
    }
}
