use lazy_static::lazy_static;

use crate::component::wire_type::{ConstValue, Signature, WireType};

pub(crate) type Param = f64;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Op {
    H,
    T,
    S,
    X,
    Y,
    Z,
    Tadj,
    Sadj,
    CX,
    ZZMax,
    Reset,
    Input,
    Output,
    Noop(WireType),
    Measure,
    Barrier,
    AngleAdd,
    AngleMul,
    AngleNeg,
    QuatMul,
    Copy { n_copies: u32, typ: WireType },
    Const(ConstValue),
    RxF64,
    RzF64,
    TK1,
    Rotation,
    ToRotation,
    Xor,
    Select(WireType),
}

impl PartialEq for Op {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Noop(l0), Self::Noop(r0)) => l0 == r0,
            (
                Self::Copy {
                    n_copies: l_n_copies,
                    typ: l_typ,
                },
                Self::Copy {
                    n_copies: r_n_copies,
                    typ: r_typ,
                },
            ) => l_n_copies == r_n_copies && l_typ == r_typ,
            (Self::Const(l0), Self::Const(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Default for Op {
    fn default() -> Self {
        Self::Noop(WireType::Qubit)
    }
}
lazy_static! {
    static ref ONEQBSIG: Signature = Signature::new_linear(vec![WireType::Qubit]);
}
lazy_static! {
    static ref TWOQBSIG: Signature = Signature::new_linear(vec![WireType::Qubit, WireType::Qubit]);
}

pub fn approx_eq(x: f64, y: f64, modulo: u32, tol: f64) -> bool {
    let modulo = f64::from(modulo);
    let x = (x - y) / modulo;

    let x = x - x.floor();

    let r = modulo * x;

    r < tol || r > modulo - tol
}

fn binary_op(typ: WireType) -> Signature {
    Signature::new_nonlinear(vec![typ, typ], vec![typ])
}

impl Op {
    pub fn is_one_qb_gate(&self) -> bool {
        matches!(self.signature().linear[..], [WireType::Qubit])
    }

    pub fn is_two_qb_gate(&self) -> bool {
        matches!(
            self.signature().linear[..],
            [WireType::Qubit, WireType::Qubit]
        )
    }

    pub fn is_pure_classical(&self) -> bool {
        self.signature().purely_classical()
    }

    pub fn signature(&self) -> Signature {
        match self {
            Op::Noop(typ) => Signature::new_linear(vec![*typ]),
            Op::H | Op::Reset | Op::T | Op::S | Op::Tadj | Op::Sadj | Op::X | Op::Y | Op::Z => {
                ONEQBSIG.clone()
            }
            Op::CX | Op::ZZMax => TWOQBSIG.clone(),
            Op::Measure => Signature::new_linear(vec![WireType::Qubit, WireType::LinearBit]),
            Op::AngleAdd | Op::AngleMul => binary_op(WireType::Angle),
            Op::QuatMul => binary_op(WireType::Quat64),
            Op::AngleNeg => Signature::new_nonlinear(vec![WireType::Angle], vec![WireType::Angle]),
            Op::Copy { n_copies, typ } => {
                Signature::new_nonlinear(vec![*typ], vec![*typ; *n_copies as usize])
            }
            Op::Const(x) => Signature::new_nonlinear(vec![], vec![x.get_type()]),

            Op::RxF64 | Op::RzF64 => {
                Signature::new(vec![WireType::Qubit], [vec![WireType::Angle], vec![]])
            }
            Op::TK1 => Signature::new(vec![WireType::Qubit], [vec![WireType::Angle; 3], vec![]]),
            Op::Rotation => Signature::new(vec![WireType::Qubit], [vec![WireType::Quat64], vec![]]),
            Op::ToRotation => Signature::new_nonlinear(
                vec![WireType::Angle, WireType::F64, WireType::F64, WireType::F64],
                vec![WireType::Quat64],
            ),
            Op::Xor => {
                Signature::new_nonlinear(vec![WireType::Bool, WireType::Bool], vec![WireType::Bool])
            }
            Op::Select(wt) => Signature::new_nonlinear(vec![WireType::Bool, *wt, *wt], vec![*wt]),
            _ => Default::default(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Op::H => "H",
            Op::T => "T",
            Op::S => "S",
            Op::X => "X",
            Op::Y => "Y",
            Op::Z => "Z",
            Op::Tadj => "Tadj",
            Op::Sadj => "Sadj",
            Op::CX => "CX",
            Op::ZZMax => "ZZMax",
            Op::Reset => "Reset",
            Op::Input => "Input",
            Op::Output => "Output",
            Op::Noop(_) => "Noop",
            Op::Measure => "Measure",
            Op::Barrier => "Barrier",
            Op::AngleAdd => "AngleAdd",
            Op::AngleMul => "AngleMul",
            Op::AngleNeg => "AngleNeg",
            Op::QuatMul => "QuatMul",
            Op::Copy { .. } => "Copy",
            Op::Const(_) => "Const",
            Op::RxF64 => "RxF64",
            Op::RzF64 => "RzF64",
            Op::TK1 => "TK1",
            Op::Rotation => "Rotation",
            Op::ToRotation => "ToRotation",
            Op::Xor => "Xor",
            Op::Select(_) => "Select",
        }
    }

    pub fn get_params(&self) -> Vec<Param> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "pyo3")]
    fn py_int(i: i32) -> Op {
        use crate::circuit::py_circuit::PyCustom;
        Op::Custom(Box::new(PyCustom(Python::with_gil(|py| i.into_py(py)))))
    }

    #[test]
    fn equality() {
        #[cfg(feature = "pyo3")]
        pyo3::prepare_freethreaded_python();
        let ops = [
            Op::Input,
            Op::Output,
            #[cfg(feature = "pyo3")]
            py_int(123),
            #[cfg(feature = "pyo3")]
            py_int(321),
            #[cfg(feature = "tkcxx")]
            unitary_x(),
            #[cfg(feature = "tkcxx")]
            unitary_z(),
            Op::Copy {
                n_copies: 3,
                typ: WireType::Qubit,
            },
        ];

        for o in &ops {
            assert_eq!(o, &o.clone());
        }

        for window in ops.windows(2) {
            assert!(window[0] != window[1]);
        }
    }
}
