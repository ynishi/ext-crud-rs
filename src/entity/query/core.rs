use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;

/// QueryContext is a struct that represents a query context.
/// It contains filter, sort, and pagination information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext<F: QueryField> {
    pub filter: Option<QueryExpr<F>>,
    pub sort: Option<Vec<SortExpr<F>>>,
    pub pagination: Option<Pagination>,
}

pub trait QueryField: Debug + Clone + Send + Sync + 'static {
    type Value: Serialize + for<'de> Deserialize<'de> + Debug + Clone + Send + Sync + 'static;
    fn name(&self) -> &'static str;
}

pub type Query<F> = Vec<QueryExpr<F>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryExpr<F: QueryField> {
    // Single comparison
    Comparison {
        field: F,
        op: ComparisonOperator<F::Value>,
    },
    // String comparison
    String {
        field: F,
        op: StringOperator,
    },
    // Multiple Logical conditions
    All(Vec<QueryExpr<F>>), // Multiple AND
    Any(Vec<QueryExpr<F>>), // Multiple OR
    Not(Box<QueryExpr<F>>), // NOT
    // Empty query
    Empty,
}

impl<F: QueryField> QueryExpr<F> {
    pub fn optimize(self) -> Self {
        match self {
            // remove double negation
            // NOT(NOT(a)) => a
            Self::Not(expr) => match *expr {
                Self::Not(inner) => (*inner).optimize(),
                other => Self::Not(Box::new(other.optimize())),
            },
            // Optimize logical All
            Self::All(exprs) => {
                // recursive optimization, then remove empty elements
                let optimized: Vec<_> =
                    Self::remove_empty(exprs.into_iter().map(|e| e.optimize()).collect());

                // summarize the result
                Self::all(optimized)
            }
            // Optimize logical Any
            Self::Any(exprs) => {
                let optimized: Vec<_> =
                    Self::remove_empty(exprs.into_iter().map(|e| e.optimize()).collect());

                Self::any(optimized)
            }
            _ => self,
        }
    }

    fn remove_empty(exprs: Vec<QueryExpr<F>>) -> Vec<QueryExpr<F>> {
        exprs
            .into_iter()
            .filter(|e| !matches!(e, Self::Empty))
            .collect()
    }

    // ユーティリティメソッド
    pub fn all(expressions: Vec<QueryExpr<F>>) -> Self {
        match expressions.len() {
            0 => Self::Empty,
            1 => expressions.into_iter().next().unwrap(),
            _ => Self::All(expressions),
        }
    }

    pub fn any(expressions: Vec<QueryExpr<F>>) -> Self {
        match expressions.len() {
            0 => Self::Empty,
            1 => expressions.into_iter().next().unwrap(),
            _ => Self::Any(expressions),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// analyze the query expression
    /// - len: the number of expressions(same as the number of nodes)
    pub fn len(&self) -> usize {
        self.nodes()
    }

    /// - nodes: the number of nodes
    pub fn nodes(&self) -> usize {
        match self {
            Self::Not(expr) => 1 + expr.nodes(),
            Self::All(exprs) | Self::Any(exprs) => {
                1 + exprs.iter().map(|e| e.nodes()).sum::<usize>()
            }
            Self::Comparison { .. } | Self::String { .. } | Self::Empty => 1,
        }
    }

    /// - leaves: the number of leaf nodes
    pub fn leaves(&self) -> usize {
        match self {
            Self::Not(expr) => expr.leaves(),
            Self::All(exprs) | Self::Any(exprs) => exprs.iter().map(|e| e.leaves()).sum(),
            Self::Comparison { .. } | Self::String { .. } => 1,
            Self::Empty => 0,
        }
    }

    /// - operators: the number of logical operators
    pub fn operators(&self) -> usize {
        match self {
            Self::Not(expr) => 1 + expr.operators(),
            Self::All(exprs) | Self::Any(exprs) => {
                1 + exprs.iter().map(|e| e.operators()).sum::<usize>()
            }
            Self::Comparison { .. } | Self::String { .. } | Self::Empty => 0,
        }
    }

    /// - depth: the maximum depth of the tree
    pub fn depth(&self) -> usize {
        match self {
            Self::Not(expr) => 1 + expr.depth(),
            Self::All(exprs) | Self::Any(exprs) => {
                1 + exprs.iter().map(|e| e.depth()).max().unwrap_or(0)
            }
            Self::Comparison { .. } | Self::String { .. } | Self::Empty => 1,
        }
    }

    /// - flatten_conditions: get all leaf conditions
    pub fn flatten_conditions(&self) -> Vec<&Self> {
        match self {
            Self::Not(expr) => expr.flatten_conditions(),
            Self::All(exprs) | Self::Any(exprs) => {
                exprs.iter().flat_map(|e| e.flatten_conditions()).collect()
            }
            Self::Comparison { .. } | Self::String { .. } => vec![self],
            Self::Empty => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOperator<T> {
    Eq(T),
    Gt(T),
    Lt(T),
    In(Vec<T>),
    IsNull,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StringOperator {
    Contains(String),
    StartsWith(String),
    EndsWith(String),
    ContainsInsensitive(String),
    StartsWithInsensitive(String),
    EndsWithInsensitive(String),
}

/// QueryBuilder is a struct that helps to build a query.
/// It provides a fluent API to add query expressions.
///
/// # Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use ext_crud_rs::entity::query::{QueryBuilder, QueryField, QueryExpr, StringOperator, ComparisonOperator };
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// enum UserField {
///   Age,
///   Name,
///   Email,
/// };
///
/// impl QueryField for UserField {
///   type Value = serde_json::Value;
///   fn name(&self) -> &'static str {
///     match self {
///       UserField::Age => "age",
///       UserField::Name => "name",
///       UserField::Email => "email",
///     }
///   }
/// }
///
/// let email_example_comparison = QueryExpr::String {
///   field: UserField::Email,
///   op: StringOperator::Contains("example.com".to_string()),
/// };
///
/// let email_not_null_comparison = QueryExpr::Not(Box::new(
///   QueryExpr::Comparison {
///     field: UserField::Email,
///     op: ComparisonOperator::IsNull,
///   }
/// ));
///
/// let query = QueryBuilder::default()
///   .add_comparison(UserField::Age, ComparisonOperator::Gt(serde_json::json!(20)))
///   .add_string_operation(UserField::Name, StringOperator::Contains("John".to_string()))
///   .and((email_example_comparison, email_not_null_comparison))
///   .build();
///
/// assert!(!query.is_empty());
/// ```
pub struct QueryBuilder<F: QueryField> {
    expressions: Vec<QueryExpr<F>>,
    joiner: QueryExpr<F>,
    _phantom: PhantomData<F>,
}

impl<F: QueryField> Default for QueryBuilder<F> {
    fn default() -> Self {
        Self {
            expressions: Vec::new(),
            _phantom: PhantomData,
            joiner: QueryExpr::Empty,
        }
    }
}

impl<F: QueryField> QueryBuilder<F> {
    pub fn set_joiner_all(mut self) -> Self {
        self.joiner = QueryExpr::All(Vec::new());
        self
    }

    pub fn set_joiner_any(mut self) -> Self {
        self.joiner = QueryExpr::Any(Vec::new());
        self
    }

    pub fn add_comparison(mut self, field: F, operator: ComparisonOperator<F::Value>) -> Self {
        self.expressions.push(QueryExpr::Comparison {
            field,
            op: operator,
        });
        self
    }

    pub fn add_string_operation(mut self, field: F, operator: StringOperator) -> Self {
        self.expressions.push(QueryExpr::String {
            field,
            op: operator,
        });
        self
    }

    pub fn and(self, expressions: (QueryExpr<F>, QueryExpr<F>)) -> Self {
        self.all(vec![expressions.0, expressions.1])
    }

    pub fn or(self, expressions: (QueryExpr<F>, QueryExpr<F>)) -> Self {
        self.any(vec![expressions.0, expressions.1])
    }

    pub fn all(mut self, expressions: Vec<QueryExpr<F>>) -> Self {
        self.expressions.push(QueryExpr::All(expressions));
        self
    }

    pub fn any(mut self, expressions: Vec<QueryExpr<F>>) -> Self {
        self.expressions.push(QueryExpr::Any(expressions));
        self
    }

    pub fn build(self) -> QueryExpr<F> {
        match self.joiner {
            QueryExpr::All(vs) => QueryExpr::All(vs),
            QueryExpr::Any(vs) => QueryExpr::Any(vs),
            _ => QueryExpr::All(self.expressions), // default joiner is ALL
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortExpr<F: QueryField> {
    pub field: F,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, size: 10 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    enum TestField {
        Age,
    }
    impl QueryField for TestField {
        type Value = serde_json::Value;
        fn name(&self) -> &'static str {
            match self {
                TestField::Age => "age",
            }
        }
    }

    #[test]
    fn test_optimize_all_empty_elements() {
        // when include empty
        let expr = QueryExpr::All(vec![
            QueryExpr::Empty,
            QueryExpr::Comparison {
                field: TestField::Age,
                op: ComparisonOperator::Gt(serde_json::json!(20)),
            },
        ]);
        let optimized = expr.optimize();
        assert_eq!(optimized.len(), 1);
        assert_eq!(
            optimized,
            QueryExpr::Comparison {
                field: TestField::Age,
                op: ComparisonOperator::Gt(serde_json::json!(20))
            }
        );

        // when all empty
        let expr: QueryExpr<TestField> = QueryExpr::All(vec![QueryExpr::Empty, QueryExpr::Empty]);

        assert_eq!(expr.optimize(), QueryExpr::Empty);
    }

    #[test]
    fn test_optimize_any_empty_elements() {
        // when include empty
        let expr = QueryExpr::Any(vec![
            QueryExpr::Empty,
            QueryExpr::Comparison {
                field: TestField::Age,
                op: ComparisonOperator::Gt(serde_json::json!(20)),
            },
        ]);
        let optimized = expr.optimize();
        assert_eq!(optimized.len(), 1);
        assert_eq!(
            optimized,
            QueryExpr::Comparison {
                field: TestField::Age,
                op: ComparisonOperator::Gt(serde_json::json!(20))
            }
        );

        // when any empty
        let expr: QueryExpr<TestField> = QueryExpr::Any(vec![QueryExpr::Empty, QueryExpr::Empty]);

        assert_eq!(expr.optimize(), QueryExpr::Empty);
    }

    #[test]
    fn test_analyze_query_expr() {
        let expr = QueryExpr::All(vec![
            QueryExpr::Comparison {
                field: TestField::Age,
                op: ComparisonOperator::Gt(serde_json::json!(20)),
            },
            QueryExpr::Any(vec![
                QueryExpr::Comparison {
                    field: TestField::Age,
                    op: ComparisonOperator::Gt(serde_json::json!(20)),
                },
                QueryExpr::Comparison {
                    field: TestField::Age,
                    op: ComparisonOperator::Gt(serde_json::json!(20)),
                },
            ]),
        ]);

        assert_eq!(expr.len(), 5);
        assert_eq!(expr.nodes(), 5);
        assert_eq!(expr.leaves(), 3);
        assert_eq!(expr.operators(), 2);
        assert_eq!(expr.depth(), 3);
        assert!(!expr.is_empty());
    }
}
