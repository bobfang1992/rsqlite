fn main() {
    let sql = "UPDATE Customers
    SET ContactName = 'Alfred Schmidt', City= 'Frankfurt'
    WHERE CustomerID = 1;";
    let ast = rsqlite::sql_parser::parse_sql(sql);
    if let Ok(statements) = ast {
        for statement in statements {
            println!("Statement is: {}", statement)
        }
    } else if let Err(error) = ast {
        println!("Error is: {}", error)
    }
}
