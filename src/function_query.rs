use crate::error::OrchestrionError;
use swc_core::ecma::ast::{FnDecl, FnExpr, Function};
use yaml_rust2::Yaml;

macro_rules! get_str {
    ($property:expr, $name:expr) => {
        $property[$name]
            .as_str()
            .ok_or(format!("Invalid config: '{}' must be a string", $name))?
    };
}

pub enum FunctionType {
    FunctionDeclaration,
    FunctionExpression,
    Method,
}

impl FunctionType {
    pub fn from_str(s: &str) -> Option<FunctionType> {
        match s {
            "decl" => Some(FunctionType::FunctionDeclaration),
            "expr" => Some(FunctionType::FunctionExpression),
            "method" => Some(FunctionType::Method),
            _ => None,
        }
    }
}

pub enum FunctionKind {
    Sync,
    Async,
    Generator,
    AsyncGenerator,
}

impl FunctionKind {
    pub fn is_async(&self) -> bool {
        matches!(self, FunctionKind::Async | FunctionKind::AsyncGenerator)
    }

    pub fn is_generator(&self) -> bool {
        matches!(self, FunctionKind::Generator | FunctionKind::AsyncGenerator)
    }

    pub fn matches(&self, func: &Function) -> bool {
        match self {
            FunctionKind::Sync => !func.is_async && !func.is_generator,
            FunctionKind::Async => func.is_async && !func.is_generator,
            FunctionKind::Generator => !func.is_async && func.is_generator,
            FunctionKind::AsyncGenerator => func.is_async && func.is_generator,
        }
    }

    pub fn from_str(s: &str) -> Option<FunctionKind> {
        match s {
            "sync" => Some(FunctionKind::Sync),
            "async" => Some(FunctionKind::Async),
            "generator" => Some(FunctionKind::Generator),
            "async generator" => Some(FunctionKind::AsyncGenerator),
            _ => None,
        }
    }
}

pub struct FunctionQuery {
    pub name: String,
    pub typ: FunctionType,
    pub kind: FunctionKind,
    pub index: usize,
}

impl FunctionQuery {
    fn maybe_increment_count(&self, matches_except_count: bool, count: &mut usize) -> bool {
        if matches_except_count {
            if self.index == *count {
                true
            } else {
                *count += 1;
                false
            }
        } else {
            false
        }
    }

    pub fn matches_decl(&self, func: &FnDecl, count: &mut usize) -> bool {
        let matches_except_count = matches!(self.typ, FunctionType::FunctionDeclaration)
            && self.kind.matches(&func.function)
            && func.ident.sym == self.name;
        self.maybe_increment_count(matches_except_count, count)
    }

    pub fn matches_expr(&self, func: &FnExpr, count: &mut usize, name: &str) -> bool {
        let matches_except_count = matches!(self.typ, FunctionType::FunctionExpression)
            && self.kind.matches(&func.function)
            && name == self.name;
        self.maybe_increment_count(matches_except_count, count)
    }

    pub fn matches_method(&self, func: &Function, count: &mut usize, name: &str) -> bool {
        let matches_except_count = matches!(self.typ, FunctionType::Method)
            && self.kind.matches(func)
            && name == self.name;
        self.maybe_increment_count(matches_except_count, count)
    }
}

impl TryFrom<&Yaml> for FunctionQuery {
    type Error = OrchestrionError;

    fn try_from(query: &Yaml) -> Result<Self, Self::Error> {
        let typ = get_str!(query, "type");
        let kind = get_str!(query, "kind");
        let name = get_str!(query, "name");
        let index: usize = query["index"].as_i64().unwrap_or(0).try_into().unwrap_or(0);

        Ok(FunctionQuery {
            name: name.to_string(),
            typ: FunctionType::from_str(typ).ok_or(format!(
                "Invalid config: 'type' must be one of 'decl', 'expr', or 'method', got '{typ}'"
            ))?,
            kind: FunctionKind::from_str(kind).ok_or(format!(
                "Invalid config: 'kind' must be one of 'sync', 'async', 'generator', or 'async generator', got '{kind}'"
            ))?,
            index,
        })
    }
}
