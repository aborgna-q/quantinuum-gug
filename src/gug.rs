use std::{
    any::{Any, TypeId},
    collections::HashMap, fmt::Debug,
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
};

#[derive(Clone, Default, Debug)]
pub struct Gug {
    pub(crate) graph: PortGraph,
    hierarchy: Hierarchy,

    op_types: SecondaryMap<NodeIndex, Op>,
    port_types: SecondaryMap<PortIndex, WireType>,

    node_metadata: HashMap<TypeId, SecondaryMap<NodeIndex, Box<dyn NodeMetadata>>>,
    port_metadata: HashMap<TypeId, SecondaryMap<PortIndex, Box<dyn PortMetadata>>>,
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

    /// Initialize a new node metadata component.
    /// If the metadata component already exists, this does nothing.
    pub fn register_node_metadata<T: NodeMetadata + Default>(&mut self) {
        self.node_metadata.entry(TypeId::of::<T>()).or_insert(
            SecondaryMap::with_default(Box::<T>::default()),
        );
    }

    /// Initialize a new port metadata component.
    /// If the metadata component already exists, this does nothing.
    pub fn register_port_metadata<T: PortMetadata + Default>(&mut self) {
        self.port_metadata.entry(TypeId::of::<T>()).or_insert(
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

    /// Gets a reference to the node metadata map for the given node component.
    /// Returns `None` if the metadata component has not been registered.
    pub fn node_metadata<T: NodeMetadata>(&self, node: NodeIndex) -> Option<&T> {
        let metadata = self.node_metadata.get(&TypeId::of::<T>())?;
        metadata.get(node).downcast_ref::<T>()
    }

    /// Gets a mutable reference to the node metadata map for the given node component.
    /// Returns `None` if the metadata component has not been registered.
    pub fn node_metadata_mut<T: NodeMetadata>(&mut self, node: NodeIndex) -> Option<&mut T> {
        let metadata = self.node_metadata.get_mut(&TypeId::of::<T>())?;
        metadata.get_mut(node).downcast_mut::<T>()
    }

    /// Gets a reference to the port metadata map for the given port component.
    /// Returns `None` if the metadata component has not been registered.
    pub fn port_metadata<T: PortMetadata>(&self, port: PortIndex) -> Option<&T> {
        let metadata = self.port_metadata.get(&TypeId::of::<T>())?;
        metadata.get(port).downcast_ref::<T>()
    }

    /// Gets a mutable reference to the port metadata map for the given port component.
    /// Returns `None` if the metadata component has not been registered.
    pub fn port_metadata_mut<T: PortMetadata>(&mut self, port: PortIndex) -> Option<&mut T> {
        let metadata = self.port_metadata.get_mut(&TypeId::of::<T>())?;
        metadata.get_mut(port).downcast_mut::<T>()
    }

    /// Applies a rewrite to the graph.
    pub fn apply_rewrite(mut self, rewrite: GugRewrite) -> Result<(), RewriteError> {
        // Get the open graph for the rewrites, and a gug with the additional components.
        let (rewrite, mut replacement) = rewrite.into_parts();

        let node_inserted = |old, new| {
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

pub trait NodeMetadata: Send + Sync + Debug + Any + Downcast + NodeMetadataBoxClone {}

impl_downcast!(NodeMetadata);
impl_box_clone!(NodeMetadata, NodeMetadataBoxClone);

pub trait PortMetadata: Send + Sync + Debug + Any + Downcast + PortMetadataBoxClone {}

impl_downcast!(PortMetadata);
impl_box_clone!(PortMetadata, PortMetadataBoxClone);