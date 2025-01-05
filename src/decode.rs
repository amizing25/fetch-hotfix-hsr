/// Represents the type of wire format for a field in the decoding process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    /// Variable-length integer (varint).
    VarInt = 0,
    /// 64-bit integer (i64).
    I64 = 1,
    /// Length-prefixed data.
    Len = 2,
    /// Start group.
    SGroup = 3,
    /// End group.
    EGroup = 4,
    /// 32-bit integer (i32).
    I32 = 5,
}

impl WireType {
    /// Converts a raw `u8` value into a `WireType` enum variant.
    /// Returns an error if the value does not correspond to a valid `WireType`.
    pub fn from_u8(value: u8) -> Result<Self, DecodeError> {
        match value {
            0 => Ok(WireType::VarInt),
            1 => Ok(WireType::I64),
            2 => Ok(WireType::Len),
            5 => Ok(WireType::I32),
            _ => Err(DecodeError::UnsupportedWireType(value)),
        }
    }
}

/// Contains the decoded field information from a decoding operation.
#[derive(Debug, Clone)]
pub struct Decoded {
    /// The field number (e.g., in Protobuf, the field number).
    pub field: u32,
    /// The wire type associated with this field.
    pub wire_type: WireType,
    /// Whether the field contains a nested object.
    pub is_object: bool,
    /// The value decoded from the field.
    pub value: DecodedValue,
}

/// Enum representing different types of decoded values.
#[derive(Debug, Clone)]
pub enum DecodedValue {
    /// A decoded BigInt (i128).
    BigInt(i128),
    /// A decoded buffer (raw bytes).
    Buffer(Vec<u8>),
    /// A decoded nested object.
    Nested(DecodingResult),
}

/// The result of decoding a structure, including both fields and unprocessed data.
#[derive(Debug, Clone)]
pub struct DecodingResult {
    /// A vector of decoded fields.
    pub fields: Vec<Decoded>,
    /// Any unprocessed bytes after decoding.
    pub unprocessed: Vec<u8>,
}

/// Simplified representation of a decoded field for easier usage.
#[derive(Debug, Clone)]
pub struct SimpleDecoded {
    /// The field number.
    pub field: u32,
    /// The string representation of the wire type.
    pub wire_type: String,
    /// Whether the field is a nested object.
    pub is_object: bool,
    /// The simplified value of the field.
    pub value: SimpleDecodedValue,
}

/// Simplified enum for decoded values in `SimpleDecoded`.
#[derive(Debug, Clone)]
pub enum SimpleDecodedValue {
    /// A simplified string representation of a decoded value.
    String(String),
    /// A simplified nested decoding result.
    Nested(SimpleDecodingResult),
}

impl std::fmt::Display for SimpleDecodedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn parse_buffer(input: &str) -> Option<String> {
            input
                .strip_prefix("Buffer([")
                .and_then(|start| start.strip_suffix("])"))
                .and_then(|end| {
                    end.split(", ")
                        .map(|byte_str| byte_str.parse::<u8>())
                        .collect::<Result<Vec<_>, _>>()
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                })
        }

        match self {
            SimpleDecodedValue::String(s) => {
                write!(f, "{}", parse_buffer(s).unwrap_or_else(|| s.clone()))
            }
            SimpleDecodedValue::Nested(nested) => write!(f, "{:?}", nested),
        }
    }
}

/// Represents the result of a simplified decoding process.
#[derive(Debug, Clone)]
pub struct SimpleDecodingResult {
    /// A vector of simplified decoded fields.
    pub fields: Vec<SimpleDecoded>,
}

/// A decoder responsible for parsing raw byte data into structured information.
#[derive(Debug)]
pub struct Decoder {
    data: Vec<u8>,
    idx: usize,
}

/// Errors that can occur during the decoding process.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("Unsupported wire type: {0}")]
    UnsupportedWireType(u8),
    #[error("Invalid memory access detected")]
    InvalidMemoryAccess,
}

impl Decoder {
    /// Creates a new `Decoder` instance with the given data.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, idx: 0 }
    }

    /// Reads the next byte from the data stream, advancing the index.
    pub fn next_byte(&mut self) -> Result<u8, DecodeError> {
        self.data
            .get(self.idx)
            .cloned()
            .ok_or(DecodeError::InvalidMemoryAccess)
            .map(|byte| {
                self.idx += 1;
                byte
            })
    }

    /// Reads the next variable-length integer (varint) from the data stream.
    pub fn next_varint(&mut self) -> Result<i128, DecodeError> {
        let mut value = 0_i128;
        let mut shift = 0;

        loop {
            let byte = self.next_byte()?;
            let current = (byte & 0x7F) as i128;
            value |= current << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        Ok(value)
    }

    /// Reads a specific number of bytes from the data stream.
    pub fn read(&mut self, length: usize) -> Result<Vec<u8>, DecodeError> {
        self.data
            .get(self.idx..self.idx + length)
            .map(|slice| {
                self.idx += length;
                slice.to_vec()
            })
            .ok_or(DecodeError::InvalidMemoryAccess)
    }

    /// Returns the number of remaining bytes to be decoded.
    pub fn remaining(&self) -> usize {
        self.data.len() - self.idx
    }

    /// Decodes the entire data stream into a `DecodingResult`.
    pub fn decode(&mut self) -> Result<DecodingResult, DecodeError> {
        let mut fields = Vec::new();

        while self.remaining() > 0 {
            let enc = self.next_varint()? as u32;
            let field = enc >> 3;
            let wire_type = WireType::from_u8((enc & 7) as u8)?;

            let mut value_decoded = false;
            let value = match wire_type {
                WireType::VarInt => DecodedValue::BigInt(self.next_varint()?),
                WireType::Len => {
                    let length = self.next_varint()? as usize;
                    let sub_data = self.read(length)?;
                    let mut nested_decoder = Decoder::new(sub_data.clone());
                    match nested_decoder.decode() {
                        Ok(decoded) => {
                            value_decoded = true;
                            DecodedValue::Nested(decoded)
                        }
                        Err(_) => DecodedValue::Buffer(sub_data),
                    }
                }
                WireType::I32 => DecodedValue::Buffer(self.read(4)?),
                WireType::I64 => DecodedValue::Buffer(self.read(8)?),
                _ => return Err(DecodeError::UnsupportedWireType((enc & 7) as u8)),
            };

            fields.push(Decoded {
                field,
                wire_type,
                is_object: value_decoded,
                value,
            });
        }

        Ok(DecodingResult {
            fields,
            unprocessed: self.read(self.remaining())?,
        })
    }
}

pub fn simplify(result: DecodingResult) -> SimpleDecodingResult {
    SimpleDecodingResult {
        fields: result
            .fields
            .into_iter()
            .map(|field| {
                let wire_type = wire_type_to_str(field.wire_type);
                let value = if field.is_object {
                    SimpleDecodedValue::Nested(simplify(field.value.unwrap_nested()))
                } else {
                    SimpleDecodedValue::String(format!("{:?}", field.value))
                };

                SimpleDecoded {
                    field: field.field,
                    wire_type,
                    is_object: field.is_object,
                    value,
                }
            })
            .collect(),
    }
}

fn wire_type_to_str(wire_type: WireType) -> String {
    match wire_type {
        WireType::VarInt => "varint".to_string(),
        WireType::I64 => "i64".to_string(),
        WireType::Len => "len".to_string(),
        WireType::I32 => "i32".to_string(),
        _ => "unknown".to_string(),
    }
}

impl DecodedValue {
    /// Unwraps a `DecodedValue::Nested` variant into the underlying `DecodingResult`.
    /// Panics if the value is not a `Nested` variant.
    fn unwrap_nested(self) -> DecodingResult {
        if let DecodedValue::Nested(result) = self {
            result
        } else {
            panic!("Expected a nested value")
        }
    }
}
