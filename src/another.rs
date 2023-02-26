use rslint_parser::SyntaxNode;

extern crate rslint_parser;
use rslint_parser::*;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Dictionary {
    pub operators: HashMap<String, usize>,
    pub operands: HashMap<String, usize>,
    pub properties: Vec<(String, String)>,
}

impl Dictionary {
    fn add_operator(&mut self, op: String) {
        match self.operators.get(&op) {
            None => self.operators.insert(op, 1),
            Some(n) => self.operators.insert(op, n + 1),
        };
    }

    fn add_operand(&mut self, od: String) {
        match self.operands.get(&od) {
            None => self.operands.insert(od, 1),
            Some(n) => self.operands.insert(od, n + 1),
        };
    }

    pub fn compute_properties(&mut self) {
        let op_dict = self.operators.len();
        let od_dict = self.operands.len();
        let op_total = self.operators.values().into_iter().sum::<usize>();
        let od_total = self.operands.values().into_iter().sum::<usize>();
        
        let program_dict = op_dict + od_dict;
        let program_len = op_total + od_total;
        let program_volume = program_len as f32 * (od_dict as f32).log2();
        
        self.properties = vec![
            ("Unique operators".to_string(), format!("{op_dict}")),
            ("Unique operands".to_string(), format!("{od_dict}")),
            ("Total operators".to_string(), format!("{op_total}")),
            ("Total operands".to_string(), format!("{od_total}")),
            ("Program dictionary".to_string(), format!("{program_dict}")),
            ("Program length".to_string(), format!("{program_len}")),
            ("Program volume".to_string(), format!("{program_volume}")),
        ]
    }
}

fn get_js_source() -> String {
    //std::fs::read_to_string("program.js").unwrap()
    std::fs::read_to_string("test.js").unwrap()
}

fn single_step(node: &SyntaxNode, ident: usize, dict: &mut Dictionary) {
    println!("{:width$}{node:?} = '{node}'", "", width = &ident);

    /* => */
    if node.is::<ast::ArrowExpr>() {
        dict.add_operator("=>".to_string());
    };

    /* All the `{ } blocks */
    if node.is::<ast::BlockStmt>() {
        dict.add_operator("{}".to_string());
    };

    /* If statement, with or without else blocks. */
    if node.is::<ast::IfStmt>() {
        dict.add_operator("if ...".to_string());
    };

    /* Any kind of `for` loops */
    if node.is::<ast::ForStmtInit>() {
        dict.add_operator("for ...".to_string());
    };

    /* Any kind of `for` loops */
    if node.is::<ast::WhileStmt>() {
        dict.add_operator("while ...".to_string());
    };

    /* Any kind of `for` loops */
    if node.is::<ast::DoWhileStmt>() {
        dict.add_operator("do ... while ...".to_string());
    };

    /* All the `=` signs */
    if node.is::<ast::Declarator>() {
        dict.add_operator("=".to_string());
    };

    /* All the `=` signs */
    if node.is::<ast::AssignExpr>() {
        let expr = ast::AssignExpr::cast(node.clone()).unwrap();
        let op_token = expr.op_token().unwrap().to_string();
        dict.add_operator(op_token);
    };

    /* Dots inside object.paths */
    if node.is::<ast::DotExpr>() {
        dict.add_operator(".".to_string());
    };

    /* Dots inside object.paths */
    if node.is::<ast::GroupingExpr>() {
        dict.add_operator("( )".to_string());
    };

    /* Constructor expression */
    if node.is::<ast::NewExpr>() {
        dict.add_operator("new ...".to_string());
    };

    /* Any identifiers/literals. */
    if node.is::<ast::Name>() || node.is::<ast::NameRef>() || node.is::<ast::Literal>() {
        let ident_or_lit = node.text().to_string();
        dict.add_operand(ident_or_lit);
    };

    /* Binary expressions */
    if node.is::<ast::BinExpr>() {
        let bin_expr = ast::BinExpr::cast(node.clone()).unwrap();
        let text = bin_expr.op_token().unwrap().to_string();
        dict.add_operator(text);
    }

    /* Unary expressions */
    if node.is::<ast::UnaryExpr>() {
        let un_expr = ast::UnaryExpr::cast(node.clone()).unwrap();
        let text = un_expr.op_token().unwrap().to_string();
        dict.add_operator(text);
    }

    /* Unary expressions */
    if node.is::<ast::ReturnStmt>() {
        dict.add_operator("return ...".to_string());
    }

    /* Unary expressions */
    if node.is::<ast::ReturnStmt>() {
        dict.add_operator("return ...".to_string());
    }

    /* Array subscription operator */
    if node.is::<ast::BracketExpr>() {
        dict.add_operator("[ ... ]".to_string());
    }
}

fn walker(node: &SyntaxNode, ident: usize, dict: &mut Dictionary) {
    /* Function and method's calls*/
    if node.is::<ast::CallExpr>() {
        /* Trying to extract function name */
        let call_expr = ast::CallExpr::cast(node.clone()).unwrap();
        let callee = call_expr.callee().unwrap();
        let syntax = callee.syntax();

        /* If it's anything more that just single item, we process the container it is in. */
        let func_name = match syntax.last_child() {
            None => syntax.to_owned(),
            Some(func_name) => {
                single_step(syntax, ident, dict);
                /* Callee without last node, which was supposedly method/function */
                for child in syntax.children() {
                    if child != func_name {
                        eprintln!("walking: {child:?}");
                        walker(&child, ident + 4, dict);
                    }
                }
                func_name
            }
        };

        /* Count function name as an operator */
        let mut function_name = func_name.trimmed_text().to_string();
        function_name.push_str("()");
        dict.add_operator(function_name);

        /* Process function arguments */
        for child in call_expr.arguments().unwrap().syntax().children() {
            walker(&child, ident + 4, dict);
        }

        if call_expr.type_args().is_some() {
            panic!("No typescript allowed. O_o");
        }

        return;
    };

    single_step(node, ident, dict);
    for child in node.children() {
        walker(&child, ident + 4, dict);
    }
}

pub fn process_js(source: &str) -> Dictionary {
    let syntax = rslint_parser::parse_text(source, 0).syntax();
    let mut dict = Default::default();
    walker(&syntax, 4, &mut dict);
    dict
}

mod lab2 {

    #[derive(Default)]
    pub struct Metrics{
        cyclomatic_complexity: usize,
    }
    use super::*;

    fn walker(node: &SyntaxNode, ident: usize, dict: &mut Metrics) {
        /* If statement, with or without else blocks. */
        if node.is::<ast::IfStmt>() {
        };
    }

    pub fn process_lab2(source: &str) -> Metrics {
        let syntax = rslint_parser::parse_text(source, 0).syntax();
        let mut metrics = Default::default();
        walker(&syntax, 4, &mut metrics);
        metrics
    }
}

