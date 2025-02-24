use std::path::PathBuf;

use nodejs_semver::{Range, Version};

use crate::function_query::*;

use yaml_rust2::{Yaml, YamlLoader};

macro_rules! get_str {
    ($property:expr, $name:expr) => {
        $property[$name]
            .as_str()
            .ok_or(format!("Invalid config: '{}' must be a string", $name))?
    };
}

macro_rules! get_arr {
    ($property:expr, $name:expr) => {
        $property[$name]
            .as_vec()
            .ok_or(format!("Invalid config: '{}' must be a array", $name))?
    };
}

#[derive(Clone)]
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

pub struct InstrumentationConfig {
    pub module_name: String,
    pub version_range: Range,
    pub file_path: PathBuf,
    pub function_query: FunctionQuery,
    pub operator: InstrumentationOperator,
    pub channel_name: String,
}

impl InstrumentationConfig {
    pub fn from_yaml_data(yaml_str: &str) -> Result<Vec<InstrumentationConfig>, String> {
        let docs = YamlLoader::load_from_str(yaml_str).unwrap();
        let doc = &docs[0];

        let version = doc["version"]
            .as_i64()
            .ok_or("Invalid config: 'version' must be a number")?;
        if version != 1 {
            return Err("Invalid config version".to_string());
        }

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
        let version: Version = version.parse().unwrap();
        self.module_name == module_name
            && version.satisfies(&self.version_range)
            && self.file_path == *file_path
    }
}

impl TryFrom<&Yaml> for InstrumentationConfig {
    type Error = String;

    fn try_from(instr: &Yaml) -> Result<Self, Self::Error> {
        let module_name = get_str!(instr, "module_name");
        let version_range = get_str!(instr, "version_range");
        let version_range: Range = version_range
            .parse()
            .map_err(|_| format!("Invalid version range: {}", version_range))?;
        let file_path = PathBuf::from(get_str!(instr, "file_path"));
        if instr["function_query"].as_hash().is_none() {
            return Err("Invalid config: 'function_query' must be a object".to_string());
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
