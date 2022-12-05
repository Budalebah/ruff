use nohash_hasher::IntSet;
use rustpython_ast::{
    Alias, Arg, Arguments, Boolop, Cmpop, Comprehension, Constant, Excepthandler,
    ExcepthandlerKind, Expr, ExprContext, Keyword, MatchCase, Operator, Pattern, Stmt, StmtKind,
    Unaryop, Withitem,
};

use crate::ast::visitor::Visitor;

#[derive(Debug)]
pub enum Trailer {
    Sibling,
    ClassDef,
    FunctionDef,
}

#[derive(Debug, Default)]
pub struct Block<'a> {
    pub imports: Vec<&'a Stmt>,
    pub trailer: Option<Trailer>,
}

#[derive(Debug)]
pub struct ImportTracker<'a> {
    exclusions: &'a IntSet<usize>,
    blocks: Vec<Block<'a>>,
}

impl<'a> ImportTracker<'a> {
    pub fn new(exclusions: &'a IntSet<usize>) -> Self {
        Self {
            exclusions,
            blocks: vec![Block::default()],
        }
    }

    fn track_import(&mut self, stmt: &'a Stmt) {
        let index = self.blocks.len() - 1;
        self.blocks[index].imports.push(stmt);
    }

    fn finalize(&mut self, trailer: Option<Trailer>) {
        let index = self.blocks.len() - 1;
        if !self.blocks[index].imports.is_empty() {
            self.blocks[index].trailer = trailer;
            self.blocks.push(Block::default());
        }
    }

    pub fn into_iter(self) -> impl IntoIterator<Item = Block<'a>> {
        self.blocks.into_iter()
    }
}

impl<'a, 'b> Visitor<'b> for ImportTracker<'a>
where
    'b: 'a,
{
    fn visit_stmt(&mut self, stmt: &'b Stmt) {
        // Track imports.
        if matches!(
            stmt.node,
            StmtKind::Import { .. } | StmtKind::ImportFrom { .. }
        ) && !self.exclusions.contains(&stmt.location.row())
        {
            self.track_import(stmt);
        } else {
            self.finalize(Some(match &stmt.node {
                StmtKind::FunctionDef { .. } | StmtKind::AsyncFunctionDef { .. } => {
                    Trailer::FunctionDef
                }
                StmtKind::ClassDef { .. } => Trailer::ClassDef,
                _ => Trailer::Sibling,
            }));
        }

        // Track scope.
        match &stmt.node {
            StmtKind::FunctionDef { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::AsyncFunctionDef { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::ClassDef { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::For { body, orelse, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::AsyncFor { body, orelse, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::While { body, orelse, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::If { body, orelse, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::With { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::AsyncWith { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            StmtKind::Match { cases, .. } => {
                for match_case in cases {
                    self.visit_match_case(match_case);
                }
            }
            StmtKind::Try {
                body,
                handlers,
                orelse,
                finalbody,
            } => {
                for excepthandler in handlers {
                    self.visit_excepthandler(excepthandler);
                }

                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);

                for stmt in orelse {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);

                for stmt in finalbody {
                    self.visit_stmt(stmt);
                }
                self.finalize(None);
            }
            _ => {}
        }
    }

    fn visit_annotation(&mut self, _: &'b Expr) {}

    fn visit_expr(&mut self, _: &'b Expr) {}

    fn visit_constant(&mut self, _: &'b Constant) {}

    fn visit_expr_context(&mut self, _: &'b ExprContext) {}

    fn visit_boolop(&mut self, _: &'b Boolop) {}

    fn visit_operator(&mut self, _: &'b Operator) {}

    fn visit_unaryop(&mut self, _: &'b Unaryop) {}

    fn visit_cmpop(&mut self, _: &'b Cmpop) {}

    fn visit_comprehension(&mut self, _: &'b Comprehension) {}

    fn visit_excepthandler(&mut self, excepthandler: &'b Excepthandler) {
        let ExcepthandlerKind::ExceptHandler { body, .. } = &excepthandler.node;
        for stmt in body {
            self.visit_stmt(stmt);
        }
        self.finalize(None);
    }

    fn visit_arguments(&mut self, _: &'b Arguments) {}

    fn visit_arg(&mut self, _: &'b Arg) {}

    fn visit_keyword(&mut self, _: &'b Keyword) {}

    fn visit_alias(&mut self, _: &'b Alias) {}

    fn visit_withitem(&mut self, _: &'b Withitem) {}

    fn visit_match_case(&mut self, match_case: &'b MatchCase) {
        for stmt in &match_case.body {
            self.visit_stmt(stmt);
        }
        self.finalize(None);
    }

    fn visit_pattern(&mut self, _: &'b Pattern) {}
}
