use cgmath::num_traits::ToPrimitive;
use num_rational::Rational64;
use std::{
    cmp::max,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[non_exhaustive]
pub enum WireType {
    Qubit,
    LinearBit,
    Bool,
    I64,
    F64,
    Quat64,
    Angle,
    /// A wire that carries no explicit information, but represents restricts
    /// the ordering of operations due to external side effects.
    SideEffects,
}

impl Default for WireType {
    fn default() -> Self {
        Self::Qubit
    }
}

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Clone, Default)]
pub struct Signature {
    pub linear: Vec<WireType>,
    pub nonlinear: [Vec<WireType>; 2],
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Signature {
    pub fn len(&self) -> usize {
        self.linear.len() + max(self.nonlinear[0].len(), self.nonlinear[1].len())
    }

    pub fn is_empty(&self) -> bool {
        self.linear.is_empty() && self.nonlinear[0].is_empty() && self.nonlinear[1].is_empty()
    }

    pub fn purely_linear(&self) -> bool {
        self.nonlinear[0].is_empty() && self.nonlinear[1].is_empty()
    }

    pub fn purely_classical(&self) -> bool {
        !self
            .linear
            .iter()
            .chain(self.nonlinear[0].iter())
            .chain(self.nonlinear[1].iter())
            .any(|typ| matches!(typ, WireType::Qubit))
    }

    /// Returns the number of input and output ports for this signature.
    pub fn num_ports(&self) -> (usize, usize) {
        (
            self.linear.len() + self.nonlinear[0].len(),
            self.linear.len() + self.nonlinear[1].len(),
        )
    }
}

impl Signature {
    pub fn new(linear: Vec<WireType>, nonlinear: [Vec<WireType>; 2]) -> Self {
        Self { linear, nonlinear }
    }

    pub fn new_linear(linear: Vec<WireType>) -> Self {
        Self {
            linear,
            nonlinear: [vec![], vec![]],
        }
    }

    pub fn new_nonlinear(inputs: Vec<WireType>, outputs: Vec<WireType>) -> Self {
        Self {
            linear: vec![],
            nonlinear: [inputs, outputs],
        }
    }

    pub fn inputs(&self) -> impl Iterator<Item = &WireType> {
        self.linear.iter().chain(self.nonlinear[0].iter())
    }

    pub fn outputs(&self) -> impl Iterator<Item = &WireType> {
        self.linear.iter().chain(self.nonlinear[1].iter())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "pyo3", pyclass(name = "Rational"))]
pub struct Rational(pub Rational64);

impl From<Rational64> for Rational {
    fn from(r: Rational64) -> Self {
        Self(r)
    }
}
// angle is contained value * pi in radians
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum AngleValue {
    F64(f64),
    Rational(Rational),
}

impl AngleValue {
    fn binary_op<F: FnOnce(f64, f64) -> f64, G: FnOnce(Rational64, Rational64) -> Rational64>(
        self,
        rhs: Self,
        opf: F,
        opr: G,
    ) -> Self {
        match (self, rhs) {
            (AngleValue::F64(x), AngleValue::F64(y)) => AngleValue::F64(opf(x, y)),
            (AngleValue::F64(x), AngleValue::Rational(y))
            | (AngleValue::Rational(y), AngleValue::F64(x)) => {
                AngleValue::F64(opf(x, y.0.to_f64().unwrap()))
            }
            (AngleValue::Rational(x), AngleValue::Rational(y)) => {
                AngleValue::Rational(Rational(opr(x.0, y.0)))
            }
        }
    }

    fn unary_op<F: FnOnce(f64) -> f64, G: FnOnce(Rational64) -> Rational64>(
        self,
        opf: F,
        opr: G,
    ) -> Self {
        match self {
            AngleValue::F64(x) => AngleValue::F64(opf(x)),
            AngleValue::Rational(x) => AngleValue::Rational(Rational(opr(x.0))),
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            AngleValue::F64(x) => *x,
            AngleValue::Rational(x) => x.0.to_f64().expect("Floating point conversion error."),
        }
    }

    pub fn radians(&self) -> f64 {
        self.to_f64() * std::f64::consts::PI
    }
}

impl Add for AngleValue {
    type Output = AngleValue;

    fn add(self, rhs: Self) -> Self::Output {
        self.binary_op(rhs, |x, y| x + y, |x, y| x + y)
    }
}

impl Sub for AngleValue {
    type Output = AngleValue;

    fn sub(self, rhs: Self) -> Self::Output {
        self.binary_op(rhs, |x, y| x - y, |x, y| x - y)
    }
}

impl Mul for AngleValue {
    type Output = AngleValue;

    fn mul(self, rhs: Self) -> Self::Output {
        self.binary_op(rhs, |x, y| x * y, |x, y| x * y)
    }
}

impl Div for AngleValue {
    type Output = AngleValue;

    fn div(self, rhs: Self) -> Self::Output {
        self.binary_op(rhs, |x, y| x / y, |x, y| x / y)
    }
}

impl Neg for AngleValue {
    type Output = AngleValue;

    fn neg(self) -> Self::Output {
        self.unary_op(|x| -x, |x| -x)
    }
}

impl Add for &AngleValue {
    type Output = AngleValue;

    fn add(self, rhs: Self) -> Self::Output {
        self.binary_op(*rhs, |x, y| x + y, |x, y| x + y)
    }
}

impl Sub for &AngleValue {
    type Output = AngleValue;

    fn sub(self, rhs: Self) -> Self::Output {
        self.binary_op(*rhs, |x, y| x - y, |x, y| x - y)
    }
}

impl Mul for &AngleValue {
    type Output = AngleValue;

    fn mul(self, rhs: Self) -> Self::Output {
        self.binary_op(*rhs, |x, y| x * y, |x, y| x * y)
    }
}

impl Div for &AngleValue {
    type Output = AngleValue;

    fn div(self, rhs: Self) -> Self::Output {
        self.binary_op(*rhs, |x, y| x / y, |x, y| x / y)
    }
}

impl Neg for &AngleValue {
    type Output = AngleValue;

    fn neg(self) -> Self::Output {
        self.unary_op(|x| -x, |x| -x)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "pyo3", pyclass(name = "Quaternion"))]
pub struct Quat(pub cgmath::Quaternion<f64>);

impl From<cgmath::Quaternion<f64>> for Quat {
    fn from(q: cgmath::Quaternion<f64>) -> Self {
        Self(q)
    }
}

#[cfg_attr(feature = "pyo3", derive(FromPyObject))]
#[derive(Clone, PartialEq, Debug)]
pub enum ConstValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    Angle(AngleValue),
    Quat64(Quat),
}

impl ConstValue {
    pub fn get_type(&self) -> WireType {
        match self {
            Self::Bool(_) => WireType::Bool,
            Self::I64(_) => WireType::I64,
            Self::F64(_) => WireType::F64,
            Self::Angle(_) => WireType::Angle,
            Self::Quat64(_) => WireType::Quat64,
        }
    }

    pub fn f64_angle(val: f64) -> Self {
        Self::Angle(AngleValue::F64(val))
    }
}
