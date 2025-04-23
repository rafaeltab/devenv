use super::tmux_format_variables::TmuxFormatVariable;

#[derive(Clone)]
pub enum RelationalOperator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

#[derive(Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Clone)]
pub enum TmuxFilterNode {
    Variable(TmuxFormatVariable),
    Const(String),
    RelationalOperation {
        op: RelationalOperator,
        lhs: Box<TmuxFilterNode>,
        rhs: Box<TmuxFilterNode>,
    },
    LogicalOperation {
        op: LogicalOperator,
        lhs: Box<TmuxFilterNode>,
        rhs: Box<TmuxFilterNode>,
    },
}

impl TmuxFilterNode {
    pub fn as_string(&self) -> String {
        match self {
            TmuxFilterNode::Variable(variable) => format!("#{{{}}}", variable.as_string()),
            TmuxFilterNode::Const(val) => val.to_string(),
            TmuxFilterNode::RelationalOperation { op, lhs, rhs } => match op {
                RelationalOperator::Equal => {
                    format!("#{{==:{},{}}}", lhs.as_string(), rhs.as_string())
                }
                RelationalOperator::NotEqual => {
                    format!("#{{!=:{},{}}}", lhs.as_string(), rhs.as_string())
                }
                RelationalOperator::LessThan => {
                    format!("#{{<:{},{}}}", lhs.as_string(), rhs.as_string())
                }
                RelationalOperator::GreaterThan => {
                    format!("#{{>:{},{}}}", lhs.as_string(), rhs.as_string())
                }
                RelationalOperator::LessThanOrEqualTo => {
                    format!("#{{<=:{},{}}}", lhs.as_string(), rhs.as_string())
                }
                RelationalOperator::GreaterThanOrEqualTo => {
                    format!("#{{>=:{},{}}}", lhs.as_string(), rhs.as_string())
                }
            },
            TmuxFilterNode::LogicalOperation { op, lhs, rhs } => match op {
                LogicalOperator::And => format!("#{{&&:{},{}}}", lhs.as_string(), rhs.as_string()),
                LogicalOperator::Or => format!("#{{||:{},{}}}", lhs.as_string(), rhs.as_string()),
            },
        }
    }
}

pub struct TmuxFilterAstBuilder {}

#[allow(dead_code)]
impl TmuxFilterAstBuilder {
    pub fn build<Build>(builder: Build) -> TmuxFilterNode
    where
        Build: Fn(TmuxFilterAstBuilder) -> TmuxFilterNode,
    {
        builder(TmuxFilterAstBuilder {})
    }
    pub fn eq(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::RelationalOperation {
            op: RelationalOperator::Equal,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
    pub fn neq(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::RelationalOperation {
            op: RelationalOperator::NotEqual,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
    pub fn lt(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::RelationalOperation {
            op: RelationalOperator::LessThan,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
    pub fn gt(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::RelationalOperation {
            op: RelationalOperator::GreaterThan,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn lte(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::RelationalOperation {
            op: RelationalOperator::LessThanOrEqualTo,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn gte(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::RelationalOperation {
            op: RelationalOperator::GreaterThanOrEqualTo,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn and(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::LogicalOperation {
            op: LogicalOperator::And,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn or(&self, lhs: TmuxFilterNode, rhs: TmuxFilterNode) -> TmuxFilterNode {
        TmuxFilterNode::LogicalOperation {
            op: LogicalOperator::Or,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn any(&self, filter: Vec<TmuxFilterNode>) -> TmuxFilterNode {
        if filter.is_empty() {
            return self.const_val("1");
        }
        if filter.len() == 1 {
            return filter[0].clone();
        }
        if filter.len() == 2 {
            return self.or(filter[0].clone(), filter[1].clone());
        }

        self.or(filter[0].clone(), self.any(filter[1..].to_vec()))
    }

    pub fn var(&self, variable: TmuxFormatVariable) -> TmuxFilterNode {
        TmuxFilterNode::Variable(variable)
    }

    pub fn const_val(&self, val: &str) -> TmuxFilterNode {
        TmuxFilterNode::Const(val.to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::infrastructure::tmux::tmux_format::*;

    #[test]
    fn basic() {
        let filter = TmuxFilterAstBuilder::build(|b| {
            b.eq(
                b.var(TmuxFormatVariable::WindowId),
                b.gte(b.var(TmuxFormatVariable::SessionId), b.const_val("10")),
            )
        });

        let res = filter.as_string();
        assert_eq!(res, "#{==:#{window_id},#{>=:#{session_id},10}}")
    }

    #[test]
    fn any() {
        let filter = TmuxFilterAstBuilder::build(|b| {
            b.any(vec![
                b.eq(b.var(TmuxFormatVariable::Host), b.const_val("example.com")),
                b.eq(b.var(TmuxFormatVariable::Host), b.const_val("google.com")),
                b.eq(
                    b.var(TmuxFormatVariable::Host),
                    b.const_val("microsoft.com"),
                ),
                b.eq(
                    b.var(TmuxFormatVariable::Host),
                    b.const_val("rafaeltab.com"),
                ),
            ])
        });

        let res = filter.as_string();
        assert_eq!(res, "#{||:#{==:#{host},example.com},#{||:#{==:#{host},google.com},#{||:#{==:#{host},microsoft.com},#{==:#{host},rafaeltab.com}}}}")
    }
}
