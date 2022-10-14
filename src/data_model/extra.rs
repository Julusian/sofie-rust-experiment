use super::ids::PieceInstanceId;

pub fn get_piece_control_object_id(piece_instance_id: &PieceInstanceId) -> String {
    format!("piece_group_control_{}", piece_instance_id.unprotect())
}
