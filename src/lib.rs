pub mod rsqlite {}

pub mod sql_parser {
    pub use sqlparser::ast::Statement;
    use sqlparser::dialect::SQLiteDialect;
    use sqlparser::parser::{Parser, ParserError};

    pub fn parse_sql(sql: &str) -> Result<std::vec::Vec<Statement>, ParserError> {
        let dialect = SQLiteDialect {};

        let ast = Parser::parse_sql(&dialect, sql);
        return ast;
    }
}

pub mod util {
    pub fn as_u16_be(array: &[u8; 2]) -> u16 {
        ((array[0] as u16) << 8) + ((array[1] as u16) << 0)
    }
    pub fn as_u32_be(array: &[u8; 4]) -> u32 {
        ((array[0] as u32) << 24)
            + ((array[1] as u32) << 16)
            + ((array[2] as u32) << 8)
            + ((array[3] as u32) << 0)
    }
}

pub mod db_page {
    use std::convert::TryInto;
    use std::fs::File;
    use std::io::Error as ioError;
    use std::io::Read;

    use crate::util;

    #[derive(Debug)]
    pub struct DBHeader {
        pub page_size_in_bits: u16,
        pub size_of_db_in_pages: u32,
    }

    impl DBHeader {
        pub fn from(f: &mut File) -> Result<DBHeader, ioError> {
            let length = 100;
            let mut header = vec![0u8; length];
            let count = f.read(header.as_mut_slice());

            match count {
                Ok(_) => {
                    let page_size_array: [u8; 2] = header[16..18].try_into().unwrap();
                    let page_count_array: [u8; 4] = header[28..32].try_into().unwrap();

                    let header = DBHeader {
                        page_size_in_bits: util::as_u16_be(&page_size_array),
                        size_of_db_in_pages: util::as_u32_be(&page_count_array),
                    };
                    return Ok(header);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db_page;
    use std::fs::File;
    use std::io::Read;
    use std::path;

    fn get_test_db_file_path() -> path::PathBuf {
        path::PathBuf::from("test/sql/chinbook.db")
    }

    fn get_test_db_file() -> File {
        let path = get_test_db_file_path();
        let f = File::open(&path).unwrap();
        f
    }

    #[test]
    fn read_file() {
        let mut f = get_test_db_file();
        let length = 100;
        let mut header = vec![0u8; length];

        let count = f.read(header.as_mut_slice()).unwrap();
        assert_eq!(count, length);
    }

    #[test]
    fn test_construct_db_header() {
        let mut f = get_test_db_file();
        let header = db_page::DBHeader::from(&mut f).unwrap();
        println!("{:?}", header);
        assert_eq!(header.page_size_in_bits, 4096);
        assert_eq!(header.size_of_db_in_pages, 224);
    }
}
