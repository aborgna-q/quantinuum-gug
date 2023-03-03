use portgraph::{
    substitute::{BoundedSubgraph, OpenGraph, Rewrite},
    PortGraph, PortIndex,
};

use crate::Gug;

/// A graph with explicit input and output ports.
#[derive(Clone, Default, Debug)]
pub struct OpenGug {
    /// The graph.
    pub gug: Gug,
    /// [`Direction::Incoming`] dangling ports in the graph.
    pub dangling_inputs: Vec<PortIndex>,
    /// [`Direction::Outgoing`] dangling ports in the graph.
    pub dangling_outputs: Vec<PortIndex>,
}

impl OpenGug {
    /// Extracts the internal open graph, and returns the Gug with additional components on the side.
    ///
    /// The returned Gug will have no graph information.
    pub fn into_parts(self) -> (OpenGraph, Gug) {
        let OpenGug {
            mut gug,
            dangling_inputs,
            dangling_outputs,
        } = self;
        let mut graph = PortGraph::default();
        std::mem::swap(&mut graph, &mut gug.graph);
        (
            OpenGraph {
                graph,
                dangling_inputs,
                dangling_outputs,
            },
            gug,
        )
    }
}

/// A rewrite operation that replaces a subgraph with another graph.
/// Includes the new weights for the nodes in the replacement graph.
#[derive(Debug, Clone)]
pub struct GugRewrite {
    subgraph: BoundedSubgraph,
    replacement: OpenGug,
}

impl GugRewrite {
    /// Creates a new rewrite operation.
    pub fn new(subgraph: BoundedSubgraph, replacement: OpenGug) -> Self {
        Self {
            subgraph,
            replacement,
        }
    }

    /// Extracts the internal graph rewrite, and returns the replacement Gug
    /// with additional components on the side.
    ///
    /// The returned Gug will have no graph information.
    pub fn into_parts(self) -> (Rewrite, Gug) {
        let (open_graph, replacement) = self.replacement.into_parts();
        (Rewrite::new(self.subgraph, open_graph), replacement)
    }
}
