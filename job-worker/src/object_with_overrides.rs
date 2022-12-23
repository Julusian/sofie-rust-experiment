use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectWithOverrides<T> {
    pub defaults: T,
    pub overrides: Vec<SomeOverrideOp>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SomeOverrideOp {
    // TODO
}

pub fn wrap_default_object<T>(obj: T) -> ObjectWithOverrides<T> {
    ObjectWithOverrides {
        defaults: obj,
        overrides: vec![],
    }
}
