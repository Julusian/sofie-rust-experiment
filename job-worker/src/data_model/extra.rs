use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::ids::{PieceInstanceId, ProtectedId};

pub fn get_piece_control_object_id(piece_instance_id: &PieceInstanceId) -> String {
    format!("piece_group_control_{}", piece_instance_id.unprotect())
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ITranslatableMessage {
    pub key: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespaces: Option<Vec<String>>,
}

#[derive(Clone, Copy, PartialEq, Deserialize_repr, Serialize_repr, Debug)]
#[repr(u8)]
pub enum NoteSeverity {
    WARNING = 1,
    ERROR = 2,
    INFO = 3,
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NoteBase<TOrigin> {
    #[serde(rename = "type")]
    pub _type: NoteSeverity,
    pub message: ITranslatableMessage,
    pub origin: TOrigin,
}
