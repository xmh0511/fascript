use super::func_expr::AstFuncExpr;
use crate::ast::types::array_type::AstArrayType;
use crate::ast::types::map_type::AstMapType;
use crate::ast::types::AstType;
use crossbeam::channel;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum TaskControl {
    Pause,
    Resume,
    Cancel,
    Rollback,
}

#[derive(Clone, Debug)]
pub enum TaskResult {
    ProgressFeedback(FasValue),
    Finish(FasValue),
    Canceled,
    Rolledback,
}

#[derive(Clone, Debug)]
pub struct TaskValue {
    ctrl_tx: channel::Sender<TaskControl>,
    result_rx: channel::Receiver<TaskResult>,
}

#[derive(Clone, Debug)]
pub struct TaskValueShadow {
    ctrl_rx: channel::Receiver<TaskControl>,
    result_tx: channel::Sender<TaskResult>,
}

impl TaskValue {
    pub fn create() -> (TaskValue, TaskValueShadow) {
        let (ctrl_tx, ctrl_rx) = channel::unbounded();
        let (result_tx, result_rx) = channel::unbounded();
        (
            TaskValue { ctrl_tx, result_rx },
            TaskValueShadow { ctrl_rx, result_tx },
        )
    }
}

#[derive(Clone, Debug)]
pub enum FasValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<FasValue>),
    IMap(HashMap<i64, FasValue>),
    SMap(HashMap<String, FasValue>),
    Func(Box<AstFuncExpr>),
    Task(TaskValue),
    //let (tx, rx) = channel::unbounded::<i32>();
}

impl PartialEq for FasValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => (l0 - r0).abs() <= 0.000001,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::IMap(l0), Self::IMap(r0)) => l0 == r0,
            (Self::SMap(l0), Self::SMap(r0)) => l0 == r0,
            (Self::Func(_l0), Self::Func(_r0)) => false, //*l0.func == *r0.func,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for FasValue {}

impl FasValue {
    pub fn get_type(&self) -> AstType {
        match self {
            FasValue::None => AstType::Void,
            FasValue::Bool(_) => AstType::Bool,
            FasValue::Int(_) => AstType::Int,
            FasValue::Float(_) => AstType::Float,
            FasValue::String(_) => AstType::String,
            FasValue::Array(v) => {
                let base_type = match v.first() {
                    Some(item) => item.get_type(),
                    None => AstType::None,
                };
                AstArrayType::new(base_type)
            }
            FasValue::IMap(im) => {
                let value_type = match im.values().last() {
                    Some(item) => item.get_type(),
                    None => AstType::None,
                };
                AstMapType::new(AstType::Int, value_type)
            }
            FasValue::SMap(sm) => {
                let value_type = match sm.values().last() {
                    Some(item) => item.get_type(),
                    None => AstType::None,
                };
                AstMapType::new(AstType::String, value_type)
            }
            FasValue::Func(f) => f.func.get_type(),
            FasValue::Task(_) => AstType::Task,
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            FasValue::None => "(null)".to_string(),
            FasValue::Bool(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            FasValue::Int(n) => format!("{}", n),
            FasValue::Float(f) => format!("{:.4}", f),
            FasValue::String(s) => s.to_string(),
            FasValue::Array(v) => {
                let items: Vec<String> = v.iter().map(|x| x.as_str()).collect();
                format!("[ {} ]", items.join(", "))
            }
            FasValue::IMap(im) => {
                let items: Vec<String> = im
                    .iter()
                    .map(|x| format!("{}: {}", x.0, x.1.as_str()))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
            FasValue::SMap(sm) => {
                let items: Vec<String> = sm
                    .iter()
                    .map(|x| format!("{}: {}", x.0, x.1.as_str()))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
            FasValue::Func(_) => "(func)".to_string(),
            FasValue::Task(_) => "(task)".to_string(),
        }
    }

    pub fn as_array(&self, base_type: AstType) -> Vec<FasValue> {
        match self {
            FasValue::Array(arr) => arr.iter().map(|x| x.as_type(base_type.clone())).collect(),
            _ => unreachable!(),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            FasValue::Bool(b) => b.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            FasValue::Float(f) => f.clone(),
            FasValue::Int(i) => i.clone() as f64,
            _ => unreachable!(),
        }
    }

    pub fn as_int(&self) -> i64 {
        match self {
            FasValue::Float(f) => f.round() as i64,
            FasValue::Int(i) => i.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_imap(&self) -> HashMap<i64, FasValue> {
        match self {
            FasValue::IMap(map) => map.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_smap(&self) -> HashMap<String, FasValue> {
        match self {
            FasValue::SMap(map) => map.clone(),
            _ => unreachable!(),
        }
    }

    pub fn as_type(&self, dest_type: AstType) -> FasValue {
        if self.get_type() != dest_type {
            match dest_type {
                AstType::None => FasValue::None,
                AstType::Array(arr_type) => {
                    let base_type = *arr_type.base_type.clone();
                    FasValue::Array(self.as_array(base_type))
                }
                AstType::Bool => self.as_bool().into(),
                AstType::Dynamic => self.clone(),
                AstType::Float => self.as_float().into(),
                AstType::Func(_) => todo!(),
                AstType::Index => unreachable!(),
                AstType::Int => self.as_int().into(),
                AstType::Map(_) => todo!(),
                AstType::String => self.as_str().into(),
                AstType::Tuple(_) => todo!(),
                AstType::Void => FasValue::None,
                AstType::Task => todo!(),
            }
        } else {
            self.clone()
        }
    }
}

pub trait GetAstTypeTrait {
    fn get_ast_type() -> AstType;
}

macro_rules! define_cast {
    ($type:ty, $v2t:tt, $t2v:tt) => {
        impl From<FasValue> for $type {
            fn from(v: FasValue) -> $type {
                v.$v2t()
            }
        }

        impl From<$type> for FasValue {
            fn from(v: $type) -> FasValue {
                FasValue::$t2v(v)
            }
        }

        impl GetAstTypeTrait for $type {
            fn get_ast_type() -> AstType {
                AstType::$t2v
            }
        }
    };
}

//define_cast2!((), as_void, None);
define_cast!(bool, as_bool, Bool);
define_cast!(f64, as_float, Float);
define_cast!(i64, as_int, Int);
define_cast!(String, as_str, String);

// void

impl From<FasValue> for () {
    fn from(_: FasValue) -> () {
        ()
    }
}

impl From<()> for FasValue {
    fn from(_: ()) -> FasValue {
        FasValue::None
    }
}

impl GetAstTypeTrait for () {
    fn get_ast_type() -> AstType {
        AstType::None
    }
}

// &str

impl From<&str> for FasValue {
    fn from(s: &str) -> FasValue {
        FasValue::String(s.into())
    }
}

impl GetAstTypeTrait for &str {
    fn get_ast_type() -> AstType {
        AstType::String
    }
}
