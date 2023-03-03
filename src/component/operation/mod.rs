#![allow(dead_code)]

use std::any::Any;

use downcast_rs::{impl_downcast, Downcast};

use crate::macros::impl_box_clone;

use super::wire_type::Signature;

pub mod circuit;

/// The operation type for a node in the GUG.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Op {
    /// A control flow node
    ControlFlow(ControlFlowOp),
    /// A quantum circuit operation
    Circuit(circuit::Op),
    /// An opaque operation that can be downcasted by the extensions that define it.
    Opaque(Box<dyn CustomOp>),
}

impl Op {
    pub fn name(&self) -> &str {
        match self {
            Self::ControlFlow(op) => op.name(),
            Self::Circuit(op) => op.name(),
            Self::Opaque(op) => op.name(),
        }
    }

    pub fn signature(&self) -> Signature {
        match self {
            Self::Circuit(op) => op.signature(),
            Self::Opaque(op) => op.signature(),
            _ => Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ControlFlowOp {
    /// A conditional operation
    #[non_exhaustive]
    Conditional,
    /// A loop operation
    #[non_exhaustive]
    Loop,
}

impl ControlFlowOp {
    pub fn name(&self) -> &str {
        match self {
            Self::Conditional => "Conditional",
            Self::Loop => "Loop",
        }
    }

    pub fn signature(&self) -> Option<Signature> {
        todo!()
    }
}

impl PartialEq for Op {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Opaque(l0), Self::Opaque(r0)) => l0.eq(&**r0),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Default for Op {
    fn default() -> Self {
        Self::Circuit(Default::default())
    }
}

#[derive(Debug)]
pub struct ToGUGFail;
pub trait CustomOp: Send + Sync + std::fmt::Debug + CustomOpBoxClone + Any + Downcast {
    fn name(&self) -> &str;

    fn signature(&self) -> Signature;

    // TODO: Create a separate GUG, or create a children subgraph in the GUG?
    fn to_gug(&self) -> Result<crate::Gug, ToGUGFail> {
        Err(ToGUGFail)
    }

    /// Check if two custom ops are equal, by downcasting and comparing the definitions.
    fn eq(&self, other: &dyn CustomOp) -> bool {
        let _ = other;
        false
    }
}

impl_downcast!(CustomOp);
impl_box_clone!(CustomOp, CustomOpBoxClone);
