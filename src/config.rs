use std::path::PathBuf;

use nodejs_semver::{Range, Version};

use crate::error::OrchestrionError;
use crate::function_query::FunctionQuery;
use crate::{get_arr, get_str};

use yaml_rust2::{Yaml, YamlLoader};

#[derive(Clone, Debug)]
pub enum InstrumentationOperator {
    Callback,
    Promise,
    Sync,
    Async,
}

impl InstrumentationOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstrumentationOperator::Callback => "traceCallback",
            InstrumentationOperator::Promise => "tracePromise",
            InstrumentationOperator::Sync => "traceSync",
            InstrumentationOperator::Async => "traceAsync",
        }
    }

    pub fn from_str(s: &str) -> Option<InstrumentationOperator> {
        match s {
            "traceCallback" => Some(InstrumentationOperator::Callback),
            "tracePromise" => Some(InstrumentationOperator::Promise),
            "traceSync" => Some(InstrumentationOperator::Sync),
            "traceAsync" => Some(InstrumentationOperator::Async),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct InstrumentationConfig {
    pub module_name: String,
    pub version_range: Range,
    pub file_path: PathBuf,
    pub function_query: FunctionQuery,
    pub operator: InstrumentationOperator,
    pub channel_name: String,
}

pub struct Config {
    pub instrumentations: Vec<InstrumentationConfig>,
    pub diagnostic_channel_module: String,
}

impl Config {
    pub fn from_yaml_data(yaml_str: &str) -> Result<Config, OrchestrionError> {
        let docs = YamlLoader::load_from_str(yaml_str)?;
        let doc = &docs[0];

        match doc["version"].as_i64() {
            Some(1) => 1,
            Some(_) => return Err("Invalid config version".into()),
            None => return Err("Invalid config: 'version' must be a number".into()),
        };

        let dc_module = doc["dc_module"].as_str().unwrap_or("diagnostics_channel");

        let configs = InstrumentationConfig::from_yaml(doc)?;

        Ok(Config {
            instrumentations: configs,
            diagnostic_channel_module: dc_module.to_string(),
        })
    }
}

impl InstrumentationConfig {
    pub fn from_yaml(doc: &Yaml) -> Result<Vec<InstrumentationConfig>, OrchestrionError> {
        let instrumentations = get_arr!(doc, "instrumentations");
        let mut configs = Vec::new();

        for instr in instrumentations {
            instr
                .as_hash()
                .ok_or("Invalid config: 'instrumentations' must be a array of objects")?;
            configs.push(instr.try_into()?);
        }

        Ok(configs)
    }

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

impl TryFrom<&Yaml> for InstrumentationConfig {
    type Error = OrchestrionError;

    fn try_from(instr: &Yaml) -> Result<Self, Self::Error> {
        let module_name = get_str!(instr, "module_name");
        let version_range = get_str!(instr, "version_range");
        let version_range: Range = version_range
            .parse()
            .map_err(|_| format!("Invalid version range: {version_range}"))?;
        let file_path = PathBuf::from(get_str!(instr, "file_path"));
        if instr["function_query"].as_hash().is_none() {
            return Err("Invalid config: 'function_query' must be a object".into());
        }
        let function_query = (&instr["function_query"]).try_into()?;
        let operator = InstrumentationOperator::from_str(get_str!(instr, "operator"))
            .unwrap_or(InstrumentationOperator::Sync);
        let channel_name = get_str!(instr, "channel_name");

        Ok(InstrumentationConfig {
            module_name: module_name.to_string(),
            version_range,
            file_path,
            function_query,
            operator,
            channel_name: channel_name.to_string(),
        })
    }
}
