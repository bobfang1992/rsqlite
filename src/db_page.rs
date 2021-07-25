use crate::record;
use crate::util;
use crate::varint;
use std::convert::TryInto;
use std::fmt;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::{prelude::*, SeekFrom};

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
    pub number_of_cells: u16,
    pub cell_pointer_array: Vec<u16>,
    pub raw_bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum PageCell {
    TableLeafPageCell {
        length: u64,
        row_id: u64,
        values: record::Record,
    },
}

impl PageCell {
    pub fn from_bytes(page_type: &DBPageType, bytes: &[u8]) -> Option<PageCell> {
        match page_type {
            DBPageType::TableLeafPage => {
                let mut cursor = 0;
                let (length, length_size_in_bytes) = varint::read_varint(&bytes[cursor..]);
                cursor += length_size_in_bytes;
                let (row_id, row_id_size_in_bytes) = varint::read_varint(&bytes[cursor..]);
                cursor += row_id_size_in_bytes;
                let record = record::Record::from_cell_bytes(&bytes[cursor..]);
                record.map(|r| PageCell::TableLeafPageCell {
                    length: length as u64,
                    row_id: row_id as u64,
                    values: r,
                })
            }
            _ => None,
        }
    }
}

impl fmt::Debug for DBPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DBPage {{ page_no: {:?}, page_type: {:?}, number_of_cells: {:?}, cell_pointer_arrary: {:?} }}",
            self.page_no, self.page_type, self.number_of_cells, self.cell_pointer_array
        )
    }
}

impl DBPage {
    pub fn raw_read(f: &mut File, off_set: u64, size: u16) -> Result<Vec<u8>, Error> {
        let mut page = vec![0u8; usize::from(size)];
        f.seek(SeekFrom::Start(off_set))?; // move cusor to offset
        f.read_exact(&mut page)?;
        Ok(page)
    }

    pub fn get_cell_pointer_array(
        raw_data: &[u8],
        page_type: DBPageType,
        number_of_cells: u16,
        is_first_page: bool,
    ) -> Vec<u16> {
        let mut cell_pointer_array = vec![0u16; usize::from(number_of_cells)];
        let mut start_offset = match page_type {
            DBPageType::IndexInteriorPage => 12,
            DBPageType::TableInteriorPage => 12,
            DBPageType::IndexLeafPage => 8,
            DBPageType::TableLeafPage => 8,
        };

        if is_first_page {
            start_offset += 100;
        }

        for i in 0..number_of_cells {
            let start = usize::from(start_offset + (i * 2));
            let end = usize::from(start_offset + ((i + 1) * 2));

            let cell_pointer_array_array: &[u8; 2] = raw_data[start..end].try_into().unwrap();
            let cell_pointer_array_value = util::as_u16_be(&cell_pointer_array_array);
            cell_pointer_array[usize::from(i)] = cell_pointer_array_value;
        }

        cell_pointer_array
    }

    pub fn read_page(f: &mut File, header: &DBHeader, page_no: u32) -> Result<DBPage, Error> {
        let raw_bytes = DBPage::raw_read(
            f,
            u64::from(page_no - 1) * u64::from(header.page_size_in_bytes),
            header.page_size_in_bytes,
        )?;

        let page_header_start_position = if page_no == 1 { 100 } else { 0 };

        let page_header: &[u8] =
            &raw_bytes[page_header_start_position..(page_header_start_position + 12)];

        let page_type = DBPageType::from_u8(page_header[0])?;

        let number_of_cells = util::as_u16_be(&page_header[3..5].try_into().unwrap());

        let cell_pointer_array =
            DBPage::get_cell_pointer_array(&raw_bytes, page_type, number_of_cells, page_no == 1);

        Ok(DBPage {
            page_no,
            page_type,
            number_of_cells,
            cell_pointer_array,
            raw_bytes,
        })
    }
    pub fn get_cell_length(&self, cell_no: u16) -> Result<i64, Error> {
        match self.page_type {
            DBPageType::IndexInteriorPage => Err(Error::new(
                ErrorKind::InvalidData,
                "Index interior page does not have cell length",
            )),
            DBPageType::TableInteriorPage => Err(Error::new(
                ErrorKind::InvalidData,
                "Table interior page does not have cell length",
            )),
            DBPageType::IndexLeafPage => Err(Error::new(
                ErrorKind::InvalidData,
                "Index leaf page does not have cell length",
            )),
            DBPageType::TableLeafPage => {
                let cell_length_start = self.cell_pointer_array[usize::from(cell_no)];
                let (result, _) =
                    varint::read_varint(&self.raw_bytes[usize::from(cell_length_start)..]);
                Ok(result)
            }
        }
    }

    pub fn get_cell(&self, cell_no: u16) -> Option<PageCell> {
        if cell_no >= self.number_of_cells {
            return None;
        }
        let start_pos = usize::from(self.cell_pointer_array[usize::from(cell_no)]);
        PageCell::from_bytes(&self.page_type, &self.raw_bytes[start_pos..])
    }
}
