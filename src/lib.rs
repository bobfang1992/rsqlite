pub mod rsqlite {}

pub mod sql_parser {
    pub use sqlparser::ast::Statement;
    use sqlparser::dialect::SQLiteDialect;
    use sqlparser::parser::{Parser, ParserError};

    pub fn parse_sql(sql: &str) -> Result<std::vec::Vec<Statement>, ParserError> {
        let dialect = SQLiteDialect {};
        Parser::parse_sql(&dialect, sql)
    }
}

pub mod util {
    pub fn as_u16_be(array: &[u8; 2]) -> u16 {
        ((array[0] as u16) << 8) + (array[1] as u16)
    }
    pub fn as_u32_be(array: &[u8; 4]) -> u32 {
        ((array[0] as u32) << 24)
            + ((array[1] as u32) << 16)
            + ((array[2] as u32) << 8)
            + (array[3] as u32)
    }
}

pub mod db_page {
    use std::convert::TryInto;
    use std::fmt;
    use std::fs::File;
    use std::io::Error;
    use std::io::ErrorKind;
    use std::io::Read;
    use std::io::{prelude::*, SeekFrom};

    use crate::util;

    #[derive(Debug)]
    pub struct DBHeader {
        pub page_size_in_bytes: u16,
        pub size_of_db_in_pages: u32,
    }

    impl DBHeader {
        pub fn from(f: &mut File) -> Result<DBHeader, Error> {
            let length = 100;
            let mut header = vec![0u8; length];
            let count = f.read(header.as_mut_slice());

            match count {
                Ok(_) => {
                    let page_size_array: [u8; 2] = header[16..18].try_into().unwrap();
                    let page_count_array: [u8; 4] = header[28..32].try_into().unwrap();

                    let header = DBHeader {
                        page_size_in_bytes: util::as_u16_be(&page_size_array),
                        size_of_db_in_pages: util::as_u32_be(&page_count_array),
                    };
                    Ok(header)
                }
                Err(e) => Err(e),
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum DBPageType {
        IndexInteriorPage = 0x02,
        TableInteriorPage = 0x05,
        IndexLeafPage = 0x0A,
        TableLeafPage = 0x0D,
    }

    impl DBPageType {
        pub fn from_u8(b: u8) -> Result<DBPageType, Error> {
            match b {
                0x02 => Ok(DBPageType::IndexInteriorPage),
                0x05 => Ok(DBPageType::TableInteriorPage),
                0x0A => Ok(DBPageType::IndexLeafPage),
                0x0D => Ok(DBPageType::TableLeafPage),
                _ => Err(Error::new(ErrorKind::InvalidData, "Unknown page type")),
            }
        }
    }

    pub struct DBPage {
        pub page_no: u32,
        pub page_type: DBPageType,

        pub raw_bytes: Vec<u8>,
    }

    impl fmt::Debug for DBPage {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "DBPage {{ page_no: {:?}, page_type: {:?} }}",
                self.page_no, self.page_type
            )
        }
    }

    impl DBPage {
        pub fn read_page(f: &mut File, header: &DBHeader, page_no: u32) -> Result<DBPage, Error> {
            let raw_bytes = DBPage::raw_read(
                f,
                u64::from(page_no - 1) * u64::from(header.page_size_in_bytes),
                header.page_size_in_bytes,
            )?;

            let page_header_start_position = if page_no == 1 { 100 } else { 0 };

            println!("page_header_start_positon: {}", page_header_start_position);
            let page_header: &[u8] =
                &raw_bytes[page_header_start_position..(page_header_start_position + 12)];

            let page_type = DBPageType::from_u8(page_header[0])?;

            return Ok(DBPage {
                page_no: page_no,
                page_type: page_type,
                raw_bytes: raw_bytes,
            });
        }

        pub fn raw_read(f: &mut File, off_set: u64, size: u16) -> Result<Vec<u8>, Error> {
            let mut page = vec![0u8; usize::from(size)];
            f.seek(SeekFrom::Start(off_set))?; // move cusor to offset
            f.read_exact(&mut page)?;
            return Ok(page);
        }

        pub fn first_page_size(page_size: u16) -> u16 {
            return page_size - 100;
        }

        pub fn get_page_type_from_raw_data(raw_data: &Vec<u8>) -> Result<DBPageType, Error> {
            match raw_data[0] {
                0x02 => Ok(DBPageType::IndexInteriorPage),
                0x05 => Ok(DBPageType::TableInteriorPage),
                0x0A => Ok(DBPageType::IndexLeafPage),
                0x0D => Ok(DBPageType::TableLeafPage),
                _ => Err(Error::new(ErrorKind::InvalidData, "Unknown page type")),
            }
        }

        pub fn get_number_of_cells_from_raw_data(raw_data: &Vec<u8>) -> u16 {
            let page_size_array: [u8; 2] = raw_data[3..5].try_into().unwrap();
            return util::as_u16_be(&page_size_array);
        }

        pub fn get_start_pos_of_cell_content_region_from_raw_data(raw_data: &Vec<u8>) -> u16 {
            let cell_content_pos: [u8; 2] = raw_data[5..7].try_into().unwrap();
            return util::as_u16_be(&cell_content_pos);
        }

        pub fn get_right_most_pointer_from_raw_data(raw_data: &Vec<u8>) -> u32 {
            let right_most_pointer_array: [u8; 4] = raw_data[8..12].try_into().unwrap();
            return util::as_u32_be(&right_most_pointer_array);
        }

        pub fn get_cell_pointer_array(
            raw_data: &Vec<u8>,
            page_type: DBPageType,
            number_of_cells: u16,
        ) -> Vec<u16> {
            let mut cell_pointer_array = vec![0u16; usize::from(number_of_cells)];
            let start_offset = match page_type {
                DBPageType::IndexInteriorPage => 12,
                DBPageType::TableInteriorPage => 12,
                DBPageType::IndexLeafPage => 8,
                DBPageType::TableLeafPage => 8,
            };

            for i in 0..number_of_cells {
                let start = usize::from(start_offset + (i * 2));
                let end = usize::from(start_offset + ((i + 1) * 2));

                let cell_pointer_array_array: [u8; 2] = raw_data[start..end].try_into().unwrap();
                let cell_pointer_array_value = util::as_u16_be(&cell_pointer_array_array);
                cell_pointer_array[usize::from(i)] = cell_pointer_array_value;
            }

            return cell_pointer_array;
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

    fn get_simple_db_file_path() -> path::PathBuf {
        path::PathBuf::from("test/sql/simple.db")
    }

    fn get_test_db_file() -> File {
        let path = get_test_db_file_path();
        let f = File::open(&path).unwrap();
        f
    }

    fn get_simple_db_file() -> File {
        let path = get_simple_db_file_path();
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
        assert_eq!(header.page_size_in_bytes, 4096);
        assert_eq!(header.size_of_db_in_pages, 224);
    }

    #[test]
    fn test_get_db_page() {
        let mut f = get_simple_db_file();
        let header = db_page::DBHeader::from(&mut f).unwrap();
        let db_page = db_page::DBPage::read_page(&mut f, &header, 1).unwrap();
        println!("header: {:?}", header);
        println!("first page: {:?}", db_page);
        let second_page = db_page::DBPage::read_page(&mut f, &header, 2).unwrap();
        println!("second page: {:?}", second_page);
    }

    // fn run() {
    //     let mut f = get_simple_db_file();
    //     let dbheader = db_page::DBHeader::from(&mut f).unwrap();
    //     let header =
    //         db_page::DBPage::raw_read(&mut f, 100, dbheader.page_size_in_bytes - 100).unwrap();
    //     let page_type = db_page::DBPage::get_page_type_from_raw_data(&header).unwrap();
    //     let num_cells = db_page::DBPage::get_number_of_cells_from_raw_data(&header);
    //     let start_pos_of_cell_content =
    //         db_page::DBPage::get_start_pos_of_cell_content_region_from_raw_data(&header);
    //     let right_most_pointer = db_page::DBPage::get_right_most_pointer_from_raw_data(&header);
    //     let cell_pointer_array =
    //         db_page::DBPage::get_cell_pointer_array(&header, page_type, num_cells);

    //     println!("header: {:?}", header);
    //     println!("page_type: {:?}", page_type);
    //     println!("num_cells: {:?}", num_cells);
    //     println!("start_pos_of_cell_content: {:?}", start_pos_of_cell_content);
    //     println!("right_most_pointer: {:?}", right_most_pointer);
    //     println!("cell_pointer_array: {:?}", cell_pointer_array);
    //     println!("dbheader: {:?}", dbheader)
    // }
}
