use crate::value;
use crate::varint;

#[derive(Debug)]
pub struct Record(Vec<value::Value>);

impl Record {
    pub fn from_cell_bytes(buf: &[u8]) -> Option<Record> {
        let (length_of_header_in_bytes, number_of_bytes_of_length) = varint::read_varint(buf);

        let header_start = number_of_bytes_of_length;
        let body_start = length_of_header_in_bytes as u64 as usize;
        let mut cursor = header_start;

        let mut serial_types = Vec::<u64>::new();
        while cursor < body_start {
            let (serial_type, offset) = varint::read_varint(&buf[cursor..]);
            serial_types.push(serial_type as u64);
            cursor += offset;
        }

        let mut result = Vec::<value::Value>::with_capacity(serial_types.len());
        for t in serial_types {
            let v = value::Value::new(t, &buf[cursor..]);
            result.push(v);
            cursor += value::Value::consume(t);
        }
        Some(Record(result))
    }
}
