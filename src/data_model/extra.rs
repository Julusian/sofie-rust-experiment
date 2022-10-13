pub fn get_piece_control_object_id(piece_instance_id: &str) -> String {
    format!("piece_group_control_{}", piece_instance_id)
}
