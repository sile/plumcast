use LocalNodeId;

pub trait GenerateLocalNodeId {
    fn generate_local_node_id(&mut self) -> LocalNodeId;
}
