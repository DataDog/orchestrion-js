/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (<https://www.datadoghq.com>/). Copyright 2025 Datadog, Inc.
 **/
use crate::function_query::FunctionQuery;
use nodejs_semver::{Range, SemverError, Version};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleMatcher {
    pub name: String,
    pub version_range: Range,
    pub file_path: PathBuf,
}

impl ModuleMatcher {
    /// Creates a new `ModuleMatcher` instance.
    /// # Errors
    /// Returns an error if the version range cannot be parsed.
    pub fn new(name: &str, version_range: &str, file_path: &str) -> Result<Self, SemverError> {
        Ok(Self {
            name: name.to_string(),
            version_range: Range::parse(version_range)?,
            file_path: PathBuf::from(file_path),
        })
    }

    #[must_use]
    pub fn matches(&self, module_name: &str, version: &str, file_path: &PathBuf) -> bool {
        let version: Version = match version.parse() {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to parse version {version}: {e}");
                return false;
            }
        };

        self.name == module_name
            && version.satisfies(&self.version_range)
            && self.file_path == *file_path
    }
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct InstrumentationConfig {
    pub module: ModuleMatcher,
    pub function_query: FunctionQuery,
    pub channel_name: String,
}

impl InstrumentationConfig {
    #[must_use]
    pub fn new(module: ModuleMatcher, function_query: FunctionQuery, channel_name: &str) -> Self {
        Self {
            module,
            function_query,
            channel_name: channel_name.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub instrumentations: Vec<InstrumentationConfig>,
    pub dc_module: String,
}

impl Config {
    #[must_use]
    pub fn new(instrumentations: Vec<InstrumentationConfig>, dc_module: Option<String>) -> Self {
        Self {
            instrumentations,
            dc_module: dc_module.unwrap_or_else(|| "diagnostics_channel".to_string()),
        }
    }

    #[must_use]
    pub fn new_single(instrumentation: InstrumentationConfig) -> Self {
        Self::new(vec![instrumentation], None)
    }
}

impl InstrumentationConfig {
    #[must_use]
    pub fn matches(&self, module_name: &str, version: &str, file_path: &PathBuf) -> bool {
        self.module.matches(module_name, version, file_path)
    }
}
