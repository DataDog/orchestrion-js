/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (<https://www.datadoghq.com>/). Copyright 2025 Datadog, Inc.
 **/
use crate::function_query::FunctionQuery;
use nodejs_semver::{Range, SemverError, Version};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum CodeMatcher {
    Dependency {
        name: String,
        version_range: Range,
        relative_path: PathBuf,
    },
    AbsolutePaths {
        absolute_paths: Vec<PathBuf>,
    },
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: Version,
    pub relative_path: PathBuf,
}

impl CodeMatcher {
    /// Creates a new `Dependency` code matcher.
    /// # Errors
    /// Returns an error if the version range cannot be parsed.
    pub fn dependency(
        name: &str,
        version_range: &str,
        relative_path: &str,
    ) -> Result<Self, SemverError> {
        Ok(CodeMatcher::Dependency {
            name: name.to_string(),
            version_range: Range::parse(version_range)?,
            relative_path: PathBuf::from(relative_path),
        })
    }

    #[must_use]
    pub fn absolute_paths(absolute_paths: Vec<PathBuf>) -> Self {
        CodeMatcher::AbsolutePaths { absolute_paths }
    }

    #[must_use]
    pub fn matches(&self, absolute_path: &PathBuf, dependency: Option<&Dependency>) -> bool {
        match self {
            CodeMatcher::Dependency {
                name,
                version_range,
                relative_path,
            } => {
                if let Some(dependency) = dependency {
                    *name == dependency.name
                        && dependency.version.satisfies(version_range)
                        && *relative_path == dependency.relative_path
                } else {
                    false
                }
            }
            CodeMatcher::AbsolutePaths { absolute_paths } => absolute_paths.contains(absolute_path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstrumentationConfig {
    pub channel_name: String,
    pub code_matcher: CodeMatcher,
    pub function_query: FunctionQuery,
}

impl InstrumentationConfig {
    #[must_use]
    pub fn new(
        channel_name: &str,
        code_matcher: CodeMatcher,
        function_query: FunctionQuery,
    ) -> Self {
        Self {
            channel_name: channel_name.to_string(),
            code_matcher,
            function_query,
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
    pub fn matches(&self, absolute_path: &PathBuf, dependency: Option<&Dependency>) -> bool {
        self.code_matcher.matches(absolute_path, dependency)
    }
}
