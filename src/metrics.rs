use rslint_parser::SyntaxNode;

extern crate rslint_parser;
use rslint_parser::ast::Pattern;
use rslint_parser::*;
use std::collections::HashMap;

#[derive(Hash, Debug, Clone, Copy)]
enum ChepinType {
    P = 1,
    M = 2,
    C = 3,
    T = 4,
}

fn upgrade_rank(rank: ChepinType, new_rank: ChepinType) -> ChepinType {
    if (new_rank as usize) > (rank as usize) {
        new_rank
    } else {
        rank
    }
}

#[derive(Hash, Debug)]
pub struct IdentProperties {
    ctype: ChepinType,
    spen: usize,
    used_in: Vec<String>,
}

#[derive(Hash, Debug)]
enum ScopeType {
    Block,
    ControllCondition,
    Assignment(String),
}

#[derive(Default, Debug)]
pub struct Dictionary {
    if_depth: usize,
    switch_djilb_cli: usize,
    cur_scope: Vec<ScopeType>,
    operators_count: usize,

    pub max_if_depth: usize,
    pub operators: HashMap<String, usize>,
    pub operands: HashMap<String, usize>,
    pub identifiers: HashMap<String, IdentProperties>,
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

    fn add_identifier(&mut self, ident: String) {
        let scope = self.cur_scope.last().expect("We are in a scope");
        let new_ctype = match &scope {
            ScopeType::Block => ChepinType::T,
            ScopeType::Assignment(ident_left) => {
                let props = self.identifiers.get_mut(ident_left).unwrap();
                props.used_in.push(ident.clone());
                ChepinType::M
            }
            ScopeType::ControllCondition => ChepinType::C,
        };

        match self.identifiers.get_mut(&ident) {
            None => {
                self.identifiers.insert(
                    ident,
                    IdentProperties {
                        ctype: new_ctype,
                        spen: 0,
                        used_in: Vec::new(),
                    },
                );
            }
            Some(IdentProperties { ctype, spen, .. }) => {
                *ctype = upgrade_rank(*ctype, new_ctype);
                *spen += 1;
            }
        };
    }

    pub fn compute_properties(&mut self) {
        let op_dict = self.operators.len();
        let od_dict = self.operands.len();
        let op_total: usize = self.operators.values().sum();
        let od_total: usize = self.operands.values().sum();

        let program_dict = op_dict + od_dict;
        let program_len = op_total + od_total;
        let program_volume = program_len as f32 * (od_dict as f32).log2();
        let amount_of_ifs = self.operators.get("if ...").unwrap_or(&0) + self.switch_djilb_cli;

        let mut if_saturation = amount_of_ifs as f32 / self.operators_count as f32;
        if if_saturation.is_nan() {
            if_saturation = 0.0;
        }
        let max_if_depth = self.max_if_depth;

        self.properties = vec![
            //("Unique operators".to_string(), format!("{op_dict}")),
            //("Unique operands".to_string(), format!("{od_dict}")),
            //("Total operators".to_string(), format!("{op_total}")),
            //("Total operands".to_string(), format!("{od_total}")),
            //("Program dictionary".to_string(), format!("{program_dict}")),
            //("Program length".to_string(), format!("{program_len}")),
            //("Program volume".to_string(), format!("{program_volume}")),
            (
                "Program statements: ".to_string(),
                format!("{}", self.operators_count),
            ),
            (
                "Djilb CL\n(amount of if's)".to_string(),
                format!("{amount_of_ifs}"),
            ),
            (
                "Djilb cl\n(if saturation)".to_string(),
                format!("{if_saturation}"),
            ),
            (
                "Djilb CLI\n(max if depth)".to_string(),
                format!("{max_if_depth}"),
            ),
        ]
    }
}

fn single_step(node: &SyntaxNode, ident: usize, dict: &mut Dictionary) {
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
        dict.operators_count += 1;
        eprintln!("{: <1$}{:?}", node, ident)
    };

    /* Any kind of `for` loops */
    if node.is::<ast::ForStmtInit>() {
        dict.add_operator("for ...".to_string());
        dict.operators_count += 2;
        eprintln!("{: <1$}{:?}", node, ident)
    };

    /* Any kind of `for` loops */
    if node.is::<ast::WhileStmt>() {
        dict.add_operator("while ...".to_string());
        dict.operators_count += 1;
        eprintln!("{: <1$}{:?}", node, ident)
    };

    /* Any kind of `for` loops */
    if node.is::<ast::DoWhileStmt>() {
        dict.add_operator("do ... while ...".to_string());
        dict.operators_count += 1;
        eprintln!("{: <1$}{:?}", node, ident)
    };

    /* All the `=` signs */
    if node.is::<ast::Declarator>() {
        dict.add_operator("=".to_string());
        dict.operators_count += 1;
        eprintln!("{: <1$}{:?}", node, ident)
    };

    /* All the `=` signs */
    if node.is::<ast::AssignExpr>() {
        let expr = ast::AssignExpr::cast(node.clone()).unwrap();
        let op_token = expr.op_token().unwrap().to_string();
        dict.add_operator(op_token);
        dict.operators_count += 1;
        eprintln!("{: <1$}{:?}", node, ident)
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

    /* Identifiers */
    if node.is::<ast::Name>() || node.is::<ast::NameRef>() {
        let ident = node.text().to_string();
        dict.add_identifier(ident);
    }

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
        dict.operators_count += 1;
        eprintln!("{: <1$}{:?}", node, ident)
    }

    /* Unary expressions */
    if node.is::<ast::ThrowStmt>() {
        dict.add_operator("return ...".to_string());
        dict.operators_count += 1;
        eprintln!("{: <1$}{:?}", node, ident)
    }

    /* Array subscription operator */
    if node.is::<ast::BracketExpr>() {
        dict.add_operator("[ ... ]".to_string());
    }
}

fn walker(node: &SyntaxNode, ident: usize, dict: &mut Dictionary) {
    /* Function and method's calls*/
    if node.is::<ast::CallExpr>() {
        if let ScopeType::Block = dict.cur_scope.last().unwrap() {
            dict.operators_count += 1;
            eprintln!("{: <1$}{:?}", node, ident)
        }

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

        return;
    };

    let mut did_enter_scope: bool = false;
    if node.is::<ast::IfStmt>() {
        dict.if_depth += 1;
        dict.cur_scope.push(ScopeType::ControllCondition);
        did_enter_scope = true;
    } else if node.is::<ast::SwitchStmt>() {
        let stmt: ast::SwitchStmt = ast::SwitchStmt::cast(node.clone()).unwrap();
        let cases_count = stmt.cases().count();
        dict.switch_djilb_cli += cases_count - 1;
        dict.if_depth += cases_count - 1;

        dict.cur_scope.push(ScopeType::ControllCondition);
        did_enter_scope = true;
    } else if node.is::<ast::WhileStmt>()
        || node.is::<ast::ForStmt>()
        || node.is::<ast::DoWhileStmt>()
        || node.is::<ast::Script>()
    {
        dict.cur_scope.push(ScopeType::Block);
        did_enter_scope = true;
    }

    /* if node.is::<ast::AssignExpr>() || node.is::<ast::Declarator>() {
        let left_side = if node.is::<ast::AssignExpr>() {
            eprintln!("ASGN => {node:?}");
            let asgn = ast::AssignExpr::cast(node.clone()).unwrap();
            // Will panic on complex assign expressions
            let lhs = asgn.lhs().unwrap();
            match lhs {
                ast::PatternOrExpr::Expr(expr) => {
                },
                ast::PatternOrExpr::Pattern(ptrn) => { }
            };
        } else {
            todo!()
        };
    }*/

    //} else if node.is::<ast::BlockStmt> || node.is::<ast::TsBoolean>{
    //}
    dict.max_if_depth = dict.max_if_depth.max(dict.if_depth);

    single_step(node, ident, dict);
    for child in node.children() {
        walker(&child, ident + 4, dict);
    }

    if node.is::<ast::IfStmt>() {
        dict.if_depth -= 1;
    } else if node.is::<ast::SwitchStmt>() {
        let stmt: ast::SwitchStmt = ast::SwitchStmt::cast(node.clone()).unwrap();
        let cases_count = stmt.cases().count();
        dict.switch_djilb_cli += cases_count - 1;
        dict.if_depth -= cases_count - 1;
    }

    if did_enter_scope {
        dict.cur_scope.pop();
    }
}

pub fn process_js(source: &str) -> Dictionary {
    let syntax = rslint_parser::parse_text(source, 0).syntax();
    let mut dict = Default::default();
    walker(&dbg!(syntax), 4, &mut dict);
    eprintln!("{:?}", dict.identifiers);
    dict
}
