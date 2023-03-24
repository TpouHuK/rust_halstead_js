use rslint_parser::SyntaxNode;

extern crate rslint_parser;
use rslint_parser::{SyntaxNodeExt, ast};

#[derive(Default, Debug)]
pub struct Metric {
    blockscheme_operators: usize,

    amount_of_ifs: usize,
    if_depth: usize,
    max_if_depth: usize,

    pub properties: Vec<(String, String)>,
}

impl Metric {
    pub fn compute_properties(&mut self) {
        #[allow(clippy::cast_precision_loss)]
        let if_saturation = self.amount_of_ifs as f32 / self.blockscheme_operators as f32;
        self.properties = vec![
            (
                "Operators count".to_owned(),
                format!("{}", self.blockscheme_operators),
            ),
            (
                "Amount of if's".to_owned(),
                format!("{}", self.amount_of_ifs),
            ),
            (
                "If saturation.".to_owned(),
                format!("{if_saturation}"),
            ),
            ("Max if depth.".to_owned(), format!("{}", self.max_if_depth)),
        ];
    }

    pub fn inc_if_depth(&mut self) {
        self.amount_of_ifs += 1;
        self.if_depth += 1;
        if self.if_depth > self.max_if_depth {
            self.max_if_depth = self.if_depth;
        }
    }

    pub fn dec_if_depth(&mut self) {
        self.if_depth -= 1;
    }
}

#[allow(clippy::wildcard_imports)]
fn process_operator(node: &SyntaxNode, metric: &mut Metric) {
    use ast::*;
    eprintln!("{node:?}");
    if node.is::<BlockStmt>() {
        process_block(node, metric);
        return;
    }
    if node.is::<EmptyStmt>() {
        return;
    }
    if node.is::<FnDecl>() {
        let stmt: ast::FnDecl = node.to();
        process_operator(stmt.body().unwrap().syntax(), metric);
        return;
    }

    metric.blockscheme_operators += 1;

    // Decision operators: If/Case
    if node.is::<IfStmt>() {
        metric.inc_if_depth();
        let stmt: IfStmt = node.to();
        process_operator(stmt.cons().unwrap().syntax(), metric);
        if let Some(else_stmt) = stmt.alt() {
            process_operator(else_stmt.syntax(), metric);
        }
        metric.dec_if_depth();
        return;
    }
    if node.is::<SwitchStmt>() {
        let stmt: SwitchStmt = node.to();
        stmt.test();
        let mut depth_inc = 0;
        for stmt in stmt.cases() {
            match stmt {
                SwitchCase::CaseClause{..} => { 
                    depth_inc += 1;
                    metric.inc_if_depth();
                    process_operator(stmt.syntax(), metric);
                }
                SwitchCase::DefaultClause{..} => {
                    process_operator(stmt.syntax(), metric);
                }
            }
        }
        metric.if_depth -= depth_inc;
        return;
    }

    // Loop operators: While/For
    if node.is::<ForStmt>() {
        let stmt: ForStmt = node.to();
        process_operator(stmt.init().unwrap().syntax(), metric);
        metric.inc_if_depth();
        process_operator(stmt.cons().unwrap().syntax(), metric);
        process_operator(stmt.update().unwrap().syntax(), metric);
        metric.dec_if_depth();
    }
    if node.is::<WhileStmt>() {
        let stmt: WhileStmt = node.to();
        metric.inc_if_depth();
        process_operator(stmt.cons().unwrap().syntax(), metric);
        metric.dec_if_depth();
    }

    // Variable declaration/Assignment
    if node.is::<Declarator>() {}
    if node.is::<AssignExpr>() {}

    // Function call
    if node.is::<CallExpr>() {}
}

fn process_block(node: &SyntaxNode, metric: &mut Metric) {
    for node in node.children() {
        process_operator(&node, metric);
    }
}

pub fn process_js(source: &str) -> Metric {
    let syntax = rslint_parser::parse_text(source, 0).syntax();
    let mut metric = Metric::default();
    eprintln!("{syntax:?}");
    process_block(&syntax, &mut metric);
    metric
}
