use crate::*;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub struct InstrumentationMatcher(Instrumentor);

#[wasm_bindgen]
impl InstrumentationMatcher {
    #[wasm_bindgen(js_name = "getTransformer")]
    pub fn get_transformer(
        &mut self,
        module_name: &str,
        version: &str,
        file_path: &str,
    ) -> Option<Transformer> {
        let instrumentations =
            self.0
                .get_matching_instrumentations(module_name, version, &PathBuf::from(file_path));

        if instrumentations.has_instrumentations() {
            Some(Transformer(instrumentations))
        } else {
            None
        }
    }
}

#[wasm_bindgen]
pub struct Transformer(InstrumentationVisitor);

#[wasm_bindgen]
impl Transformer {
    #[wasm_bindgen]
    pub fn transform(&mut self, contents: &str, is_module: bool) -> Result<String, JsValue> {
        let is_module = IsModule::Bool(is_module);
        self.0
            .transform(contents, is_module)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub fn create(
    configs: Vec<InstrumentationConfig>,
    dc_module: Option<String>,
) -> InstrumentationMatcher {
    InstrumentationMatcher(Instrumentor::new(Config::new(configs, dc_module)))
}
