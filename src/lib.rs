pub mod db_page;
pub mod record;
pub mod sql_parser;
pub mod util;
pub mod value;
pub mod varint;

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
        let header = self::db_page::DBHeader::from(&mut f).unwrap();
        assert_eq!(header.page_size_in_bytes, 4096);
        assert_eq!(header.size_of_db_in_pages, 224);
    }

    #[test]
    fn test_get_db_page() {
        let mut f = get_simple_db_file();
        let header = self::db_page::DBHeader::from(&mut f).unwrap();
        let first_page = self::db_page::DBPage::read_page(&mut f, &header, 1).unwrap();
        let cell_length = first_page.get_cell_length(0);

        println!("header: {:?}", header);
        println!("first page: {:?}", first_page);
        println!("all bytes: {:?}", &first_page.raw_bytes);
        println!("cell length: {:?}", cell_length);

        let cell_0 = first_page.get_cell(0);
        println!("cell 0: {:?}", cell_0);

        // let second_page = self::db_page::DBPage::read_page(&mut f, &header, 2).unwrap();
        // let second_cell_length = second_page.get_cell_length(0);
        // println!("second page: {:?}", second_page);
        // println!("Second page first cell length: {:?}", second_cell_length);
        // println!(
        //     "Second page second cell length: {:?}",
        //     second_page.get_cell_length(1)
        // );
        // println!("all bytes second: {:?}", second_page.raw_bytes);

        // let r = record::Record::from_cell_bytes(&second_page.raw_bytes[1022..]);
        // println!("record: {:?}", r);

        // let third_page = self::db_page::DBPage::read_page(&mut f, &header, 3).unwrap();
        // let third_cell_length = third_page.get_cell_length(0);
        // println!("second page: {:?}", third_page);
        // println!("Second page first cell length: {:?}", third_cell_length);
        // println!(
        //     "Second page second cell length: {:?}",
        //     third_page.get_cell_length(1)
        // );
        // println!("all bytes second: {:?}", third_page.raw_bytes);

        // let r3 = record::Record::from_cell_bytes(&third_page.raw_bytes[1013..]);
        // println!("record: {:?}", r3);

        // let r2 = record::Record::from_cell_bytes(&third_page.raw_bytes[1020..]);
        // println!("record: {:?}", r2);

        // let cell_0 = third_page.get_cell(0);
        // println!("cell 0: {:?}", cell_0);
        // let cell_1 = third_page.get_cell(1);
        // println!("cell 1: {:?}", cell_1);
        // let cell_2 = third_page.get_cell(2);
        // println!("cell 2: {:?}", cell_2);
    }
}
