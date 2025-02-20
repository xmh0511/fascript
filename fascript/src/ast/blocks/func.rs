use crate::ast::exprs::AstExpr;
use crate::ast::stmts::AstStmt;
use crate::ast::types::func_type::AstFuncType;
use crate::ast::types::AstType;
use crate::ast::ParseExt;
use crate::ast::PestApiExt;
use crate::ast::Rule;
use crate::FasCallable;
use std::fmt::Debug;

pub enum AstFunc {
    NativeFunc(Box<dyn FasCallable>),
    FasFunc(FasFunc),
    FasTask(FasTask),
}

impl AstFunc {
    pub fn parse_lambda(root: pest::iterators::Pair<'_, Rule>) -> Self {
        AstFunc::FasFunc(FasFunc::parse_lambda(root))
    }

    pub fn get_name(&self) -> String {
        match self {
            AstFunc::NativeFunc(_) => "".to_string(),
            AstFunc::FasFunc(func) => func.name.clone(),
            AstFunc::FasTask(task) => task.name.clone(),
        }
    }

    pub fn get_type(&self) -> AstType {
        AstFuncType::new(match self {
            AstFunc::NativeFunc(nfunc) => nfunc.get_arg_count(),
            AstFunc::FasFunc(func) => func.arg_names.len(),
            AstFunc::FasTask(task) => task.arg_names.len(),
        })
    }
}

impl ParseExt for AstFunc {
    fn parse(root: pest::iterators::Pair<'_, Rule>) -> Self {
        match root.as_rule() {
            Rule::FuncStmt => AstFunc::FasFunc(FasFunc::parse(root)),
            Rule::TaskStmt => AstFunc::FasTask(FasTask::parse(root)),
            _ => unreachable!(),
        }
    }
}

impl Clone for AstFunc {
    fn clone(&self) -> Self {
        match self {
            Self::NativeFunc(arg0) => Self::NativeFunc(arg0.clone_()),
            Self::FasFunc(arg0) => Self::FasFunc(arg0.clone()),
            Self::FasTask(arg0) => Self::FasTask(arg0.clone()),
        }
    }
}

impl Debug for AstFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeFunc(arg0) => f.debug_struct("NativeFunc").finish(),
            Self::FasFunc(arg0) => f.debug_tuple("FasFunc").field(arg0).finish(),
            Self::FasTask(arg0) => f.debug_tuple("FasTask").field(arg0).finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FasFunc {
    pub name: String,
    pub arg_names: Vec<String>,
    pub body_stmts: Vec<AstStmt>,
}

impl ParseExt for FasFunc {
    fn parse(root: pest::iterators::Pair<'_, Rule>) -> Self {
        let mut _func = FasFunc {
            name: "".to_string(),
            arg_names: vec![],
            body_stmts: vec![],
        };
        for root_item in root.into_inner() {
            match root_item.as_rule() {
                Rule::Id => _func.name = root_item.get_id(),
                Rule::Args => _func.arg_names = root_item.get_ids(),
                Rule::FuncBody => _func.body_stmts = AstStmt::parse_stmts(root_item),
                _ => unreachable!(),
            }
        }
        _func
    }
}

#[derive(Clone, Debug)]
pub struct FasTask {
    pub annotations: Vec<AstAnnoPart>,
    pub name: String,
    pub arg_names: Vec<String>,
    pub body_stmts: Vec<AstStmt>,
}

impl ParseExt for FasTask {
    fn parse(root: pest::iterators::Pair<'_, Rule>) -> Self {
        let mut _func = FasTask {
            annotations: vec![],
            name: "".to_string(),
            arg_names: vec![],
            body_stmts: vec![],
        };
        for root_item in root.into_inner() {
            match root_item.as_rule() {
                Rule::AnnoPart => _func.annotations.push(AstAnnoPart::parse(root_item)),
                Rule::Id => _func.name = root_item.get_id(),
                Rule::Args => _func.arg_names = root_item.get_ids(),
                Rule::FuncBody => _func.body_stmts = AstStmt::parse_stmts(root_item),
                _ => unreachable!(),
            }
        }
        _func
    }
}

#[derive(Clone, Debug)]
pub struct AstAnnoPart {
    pub anno_type: String,
    pub anno_expr: AstExpr,
}

unsafe impl Send for AstAnnoPart {}

impl ParseExt for AstAnnoPart {
    fn parse(root: pest::iterators::Pair<'_, Rule>) -> Self {
        let mut anno_type = "".to_string();
        let mut anno_expr = AstExpr::None;
        for root_item in root.into_inner() {
            match root_item.as_rule() {
                Rule::Id => {
                    anno_type = root_item.get_id();
                    if !vec!["pause", "resume", "degradation", "rollback", "retry"]
                        .contains(&&anno_type[..])
                    {
                        panic!()
                    }
                }
                Rule::MiddleExpr => {
                    let expr = AstExpr::parse_middle_expr(root_item);
                    if expr.0.len() + expr.2.len() > 0 {
                        panic!()
                    }
                    anno_expr = expr.1;
                }
                _ => unreachable!(),
            }
        }
        AstAnnoPart {
            anno_type,
            anno_expr,
        }
    }
}

// unsafe impl Send for FasFunc {}

impl FasFunc {
    pub fn parse_lambda(root: pest::iterators::Pair<'_, Rule>) -> Self {
        let mut _func = FasFunc {
            name: "".to_string(),
            arg_names: vec![],
            body_stmts: vec![],
        };
        for root_item in root.into_inner() {
            match root_item.as_rule() {
                Rule::Id => {
                    _func.arg_names.push(root_item.get_id());
                }
                Rule::Args => _func.arg_names = root_item.get_ids(),
                Rule::FuncBody => _func.body_stmts = AstStmt::parse_stmts(root_item),
                _ => unreachable!(),
            }
        }
        _func
    }
}
