#[macro_export]
macro_rules! get_str {
    ($property:expr, $name:expr) => {
        $property[$name]
            .as_str()
            .ok_or(format!("Invalid config: '{}' must be a string", $name))?
    };
}

#[macro_export]
macro_rules! get_arr {
    ($property:expr, $name:expr) => {
        $property[$name]
            .as_vec()
            .ok_or(format!("Invalid config: '{}' must be a array", $name))?
    };
}

#[macro_export]
macro_rules! ident {
    ($name:expr) => {
        Ident::new($name.into(), Span::default(), SyntaxContext::empty())
    };
}
