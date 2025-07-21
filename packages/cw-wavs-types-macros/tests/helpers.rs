use serde_json::Value;
use std::collections::HashSet;

#[macro_export]
macro_rules! variants_of_enum {
    ($enum:ty) => {
        extract_enum_variants_from_schema(&serde_json::to_value(schema_for!($enum)).unwrap())
    };
}

/// Extracts the variant names from a schema of an enum.
#[allow(dead_code)]
pub fn extract_enum_variants_from_schema(schema: &Value) -> HashSet<String> {
    let mut variants = HashSet::new();

    if let Some(one_of) = schema.get("oneOf").and_then(|v| v.as_array()) {
        for variant_schema in one_of {
            if let Some(properties) = variant_schema.get("properties") {
                if let Some(obj) = properties.as_object() {
                    for key in obj.keys() {
                        variants.insert(key.clone());
                    }
                }
            }
        }
    }

    variants
}
