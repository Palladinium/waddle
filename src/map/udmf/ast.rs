use std::ops::Range;

use crate::{
    map::{
        udmf::{self, CompileError, Identifier, Value},
        RawMap,
    },
    String8,
};

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub item: T,
    pub span: Range<usize>,
}

impl<T> Spanned<T> {
    pub fn wrap((item, span): (T, Range<usize>)) -> Self {
        Self { item, span }
    }
}

#[derive(Clone, Debug)]
pub struct AssignmentExpr {
    pub identifier: Spanned<Identifier>,
    pub value: Spanned<Value>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub identifier: Spanned<Identifier>,
    pub assignments: Vec<Spanned<AssignmentExpr>>,
}

#[derive(Clone, Debug)]
pub struct TranslationUnit {
    pub expressions: Vec<GlobalExpr>,
}

impl TranslationUnit {
    pub fn compile(&self, name: String8) -> Result<RawMap, Box<CompileError>> {
        udmf::compile_udmf_translation_unit(self, name)
    }
}

#[derive(Clone, Debug)]
pub enum GlobalExpr {
    AssignmentExpr(Spanned<AssignmentExpr>),
    Block(Spanned<Block>),
}
