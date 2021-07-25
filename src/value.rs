use crate::util;
use std::convert::TryInto;

#[derive(Debug)]
pub enum Value {
    Null,
    Int8([u8; 1]),
    Int16([u8; 2]),
    Int24([u8; 3]),
    Int32([u8; 4]),
    Int48([u8; 6]),
    Int64([u8; 8]),
    Float64(f64),
    Zero,
    One,
    SQLiteString(String),
}

impl Value {
    pub fn consume(serial_type: u64) -> usize {
        if serial_type >= 12 {
            if serial_type % 2 == 0 {
                return ((serial_type - 12) / 2) as usize;
            }
            if serial_type % 2 == 1 {
                return ((serial_type - 13) / 2) as usize;
            }
        }
        match serial_type {
            0x00 => 0,
            0x01 => 1,
            0x02 => 2,
            0x03 => 3,
            0x04 => 4,
            0x05 => 6,
            0x06 => 8,
            0x07 => 8,
            0x08 => 0,
            0x09 => 0,
            _ => panic!("invalid serial_type: {:?}", serial_type),
        }
    }
    pub fn new(serial_type: u64, value: &[u8]) -> Value {
        if serial_type >= 12 {
            if serial_type % 2 == 0 {
                panic!("invalid serial_type: {:?}", serial_type);
            }
            if serial_type % 2 == 1 {
                let length = ((serial_type - 13) / 2) as usize;
                let s = std::str::from_utf8(&value[..length]);
                if let Ok(ss) = s {
                    return Value::SQLiteString(ss.to_string());
                } else {
                    panic!("correputed string bytes {:?}", &value[..length]);
                }
            }
        }

        match serial_type {
            0x00 => Value::Null,
            0x01 => Value::Int8(value[0..1].try_into().unwrap()),
            0x02 => Value::Int16(value[0..2].try_into().unwrap()),
            0x03 => Value::Int24(value[0..3].try_into().unwrap()),
            0x04 => Value::Int32(value[0..4].try_into().unwrap()),
            0x05 => Value::Int48(value[0..6].try_into().unwrap()),
            0x06 => Value::Int64(value[0..8].try_into().unwrap()),
            0x07 => Value::Float64(util::as_f64_be(value[0..8].try_into().unwrap())),
            0x08 => Value::Zero,
            0x09 => Value::One,
            _ => panic!("unexpected serial type"),
        }
    }

    #[allow(clippy::all)]
    pub fn as_i64(&self) -> Option<i64> {
        let mut array: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        match self {
            Value::Int8(value) => {
                array[7] = value[0];
            }
            Value::Int16(value) => {
                array[6] = value[0];
                array[7] = value[1];
            }
            Value::Int24(value) => {
                array[5] = value[0];
                array[6] = value[1];
                array[7] = value[2];
            }
            Value::Int32(value) => {
                array[4] = value[0];
                array[5] = value[1];
                array[6] = value[2];
                array[7] = value[3];
            }
            Value::Int48(value) => {
                array[2] = value[0];
                array[3] = value[1];
                array[4] = value[2];
                array[5] = value[3];
                array[6] = value[4];
                array[7] = value[5];
            }
            Value::Int64(value) => {
                array[0] = value[0];
                array[1] = value[1];
                array[2] = value[2];
                array[3] = value[3];
                array[4] = value[4];
                array[5] = value[5];
                array[6] = value[6];
                array[7] = value[7];
            }
            __ => return None,
        };
        return Some(i64::from_be_bytes(array));
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float64(f) => Some(*f),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Int8(a), Value::Int8(b)) => a == b,
            (Value::Int16(a), Value::Int16(b)) => a == b,
            (Value::Int24(a), Value::Int24(b)) => a == b,
            (Value::Int32(a), Value::Int32(b)) => a == b,
            (Value::Int48(a), Value::Int48(b)) => a == b,
            (Value::Int64(a), Value::Int64(b)) => a == b,
            (Value::Float64(a), Value::Float64(b)) => a == b,
            (Value::Zero, Value::Zero) => true,
            (Value::One, Value::One) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() {
        let value: [u8; 0] = [];
        assert_eq!(Value::new(0, &value), Value::Null);
    }
}
