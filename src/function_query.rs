use yaml_rust2::Yaml;
use swc_core::ecma::ast::*;

macro_rules! get_str {
    ($property:expr, $name:expr) => {
        $property
            .as_str()
            .ok_or(format!("Invalid config: '{}' must be a string", $name))?
    };
}

pub enum FunctionType {
    FunctionDeclaration,
    FunctionExpression,
}

impl FunctionType {
    pub fn from_str(s: &str) -> Option<FunctionType> {
        match s {
            "decl" => Some(FunctionType::FunctionDeclaration),
            "expr" => Some(FunctionType::FunctionExpression),
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
    pub fn matches_decl(&self, func: &FnDecl, count: usize) -> bool {
        // TODO(bengl) check if it's only the count that's wrong, and somehow 
        // signal that so we can update the counter.
        matches!(self.typ, FunctionType::FunctionDeclaration)
            && self.kind.matches(&func.function)
            && func.ident.sym == self.name
            && count == self.index
    }

    pub fn matches_expr(&self, func: &FnExpr, count: usize, name: &str) -> bool {
        // TODO(bengl) check if it's only the count that's wrong, and somehow 
        // signal that so we can update the counter.
        matches!(self.typ, FunctionType::FunctionExpression)
            && self.kind.matches(&func.function)
            && name == self.name
            && count == self.index
    }
}

impl TryFrom<&Yaml> for FunctionQuery {
    type Error = String;

    fn try_from(query: &Yaml) -> Result<Self, Self::Error> {
        let typ = get_str!(query["type"], "type");
        let kind = get_str!(query["kind"], "kind");
        let name = get_str!(query["name"], "name");
        let index = query["index"].as_i64().unwrap_or(0) as usize;

        Ok(FunctionQuery {
            name: name.to_string(),
            typ: FunctionType::from_str(typ).unwrap(),
            kind: FunctionKind::from_str(kind).unwrap(),
            index,
        })
    }
}
