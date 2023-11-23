pub mod ast;
pub mod exec;
pub mod utils;

use crate::ast::exprs::value_expr::FasValue;
use ast::blocks::program::AstProgram;
use ast::FromStringExt;
use exec::task_runner::TaskRunner;
use lazy_static::lazy_static;
use std::sync::Mutex;
use utils::native_func_utils::{FasCallable, FasToWrapper};

pub struct FasRuntime {
    runner: TaskRunner,
}

impl FasRuntime {
    pub fn new() -> FasRuntime {
        FasRuntime {
            runner: TaskRunner::new(),
        }
    }

    pub async fn run(&mut self, code: &str) -> Option<FasValue> {
        let stmts = match AstProgram::from_str(code) {
            Ok(program) => program.stmts,
            Err(err) => {
                println!("parse error: {}", err);
                return None;
            }
        };
        for stmt in stmts {
            self.runner.eval_stmt(stmt);
        }
        self.runner.get_return_value()
    }

    fn set_func_impl<T: FasCallable>(&mut self, func_name: String, f: T) {
        let func = f.to_fas_value(func_name.clone());
        self.runner.set_global_value(func_name, func)
    }

    pub fn set_func<T: FasToWrapper<U>, U>(&mut self, func_name: String, f: T) {
        self.set_func_impl(func_name, f.convert());
    }
}

// lazy_static! {
//     static ref RUNTIME_INST: Mutex<FasRuntime> = Mutex::new(Fascript::new_runtime());
// }

// pub struct Fascript {}

// impl Fascript {
//     pub fn new_runtime() -> FasRuntime {
//         FasRuntime::new()
//     }

//     pub async fn set_global_value(name: String, value: FasValue) {
//         let mut runtime_inst = RUNTIME_INST.lock().await;
//         runtime_inst.set_global_value(name, value)
//     }

//     pub async fn set_global_func(
//         name: String,
//         func_impl: Box<fn(Vec<FasValue>) -> FasValue>,
//         arg_types: Vec<AstType>,
//         ret_type: AstType,
//     ) {
//         let mut runtime_inst = RUNTIME_INST.lock().await;
//         runtime_inst.set_global_func(name, func_impl, arg_types, ret_type)
//     }

//     pub async fn set_func<A, R, F: Fn(A) -> R>(func_name: String, func: F) {
//         let mut runtime_inst = RUNTIME_INST.lock().await;
//         runtime_inst.set_func(func_name, func)
//     }
// }
