pub use sqlparser::ast::Statement;
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::{Parser, ParserError};

pub fn parse_sql(sql: &str) -> Result<std::vec::Vec<Statement>, ParserError> {
    let dialect = SQLiteDialect {};
    Parser::parse_sql(&dialect, sql)
}
