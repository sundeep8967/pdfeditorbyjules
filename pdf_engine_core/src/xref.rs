use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XrefEntry {
    Free { next_free_object: u32, generation_number: u16 },
    InUse { byte_offset: u64, generation_number: u16 },
    Compressed { object_stream_num: u32, index_in_stream: u16 },
}

#[derive(Debug, Clone)]
pub struct XrefTable {
    pub entries: HashMap<u32, XrefEntry>,
    pub trailer_dict: Option<PdfDictionary>,
}

impl XrefTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            trailer_dict: None,
        }
    }
}

use crate::object::PdfDictionary;

impl XrefTable {
    pub fn trailer(&self) -> Option<&PdfDictionary> {
        self.trailer_dict.as_ref()
    }
}
