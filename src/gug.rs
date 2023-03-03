use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use downcast_rs::{impl_downcast, Downcast};
use portgraph::{
    substitute::RewriteError, Hierarchy, NodeIndex, PortGraph, PortIndex, SecondaryMap,
};

use crate::{
    component::{
        operation::Op,
        wire_type::{Signature, WireType},
    },
    macros::impl_box_clone,
    rewrite::GugRewrite,
    DebugData,
};

#[derive(Clone, Default)]
pub struct Gug {
    pub(crate) graph: PortGraph,
    hierarchy: Hierarchy,
    debug_data: SecondaryMap<NodeIndex, DebugData>,
    op_types: SecondaryMap<NodeIndex, Op>,
    node_metadata: HashMap<TypeId, SecondaryMap<NodeIndex, Box<dyn NodeMetadata>>>,

    port_types: SecondaryMap<PortIndex, WireType>,
    port_metadata: HashMap<TypeId, SecondaryMap<PortIndex, Box<dyn NodeMetadata>>>,
}

impl Gug {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(num_nodes: usize, num_edges: usize) -> Self {
        Self {
            graph: PortGraph::with_capacity(num_nodes, num_edges),
            ..Default::default()
        }
    }

    pub fn add_node_metadata<T: NodeMetadata + Default>(&mut self) {
        self.node_metadata.insert(
            TypeId::of::<T>(),
            SecondaryMap::with_default(Box::<T>::default()),
        );
    }

    pub fn optype(&self, node: NodeIndex) -> &Op {
        &self.op_types[node]
    }

    pub fn set_optype(&mut self, node: NodeIndex, op: Op) {
        let (_input_ports, _output_ports) = op.signature().num_ports();
        // TODO resize number of ports according to signature
        // self.graph.reallocate_ports(node, input_ports, output_ports);
        self.op_types[node] = op;
    }

    pub fn signature(&self, node: NodeIndex) -> Signature {
        self.optype(node).signature()
    }

    pub fn metadata<T: NodeMetadata>(&self, node: NodeIndex) -> Option<&T> {
        let metadata = self.node_metadata.get(&TypeId::of::<T>())?;
        metadata.get(node).downcast_ref::<T>()
    }

    pub fn metadata_mut<T: NodeMetadata>(&mut self, node: NodeIndex) -> Option<&mut T> {
        let metadata = self.node_metadata.get_mut(&TypeId::of::<T>())?;
        metadata.get_mut(node).downcast_mut::<T>()
    }

    /// Applies a rewrite to the graph.
    pub fn apply_rewrite(mut self, rewrite: GugRewrite) -> Result<(), RewriteError> {
        // Get the open graph for the rewrites, and a gug with the additional components.
        let (rewrite, mut replacement) = rewrite.into_parts();

        let node_inserted = |old, new| {
            std::mem::swap(&mut self.debug_data[new], &mut replacement.debug_data[old]);
            std::mem::swap(&mut self.op_types[new], &mut replacement.op_types[old]);
            for (type_id, replacement_meta) in replacement.node_metadata.iter_mut() {
                if let Some(meta) = self.node_metadata.get_mut(type_id) {
                    std::mem::swap(&mut meta[new], &mut replacement_meta[old]);
                }
            }
        };
        let port_inserted = |old, new| {
            std::mem::swap(&mut self.port_types[new], &mut replacement.port_types[old]);
            for (type_id, replacement_meta) in replacement.port_metadata.iter_mut() {
                if let Some(meta) = self.port_metadata.get_mut(type_id) {
                    std::mem::swap(&mut meta[new], &mut replacement_meta[old]);
                }
            }
        };
        rewrite.apply_with_callbacks(
            &mut self.graph,
            |_| {},
            |_| {},
            node_inserted,
            port_inserted,
            |_, _| {},
        )
    }
}

impl std::fmt::Debug for Gug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GUG")
            .field("graph", &self.graph)
            .field("hierarchy", &self.hierarchy)
            .field("debug_data", &self.debug_data)
            .field("op_types", &self.op_types)
            .finish()
    }
}

pub trait NodeMetadata: Send + Sync + Any + Downcast + NodeMetadataBoxClone {}

impl_downcast!(NodeMetadata);
impl_box_clone!(NodeMetadata, NodeMetadataBoxClone);

pub trait PortMetadata: Send + Sync + Any + Downcast + PortMetadataBoxClone {}

impl_downcast!(PortMetadata);
impl_box_clone!(PortMetadata, PortMetadataBoxClone);
