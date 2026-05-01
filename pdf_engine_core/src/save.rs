use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::PdfError;
use crate::object::ObjectId;
use crate::document::PdfDocument;

use crate::edit::serialize_object;

impl PdfDocument {
    /// Saves the document using an Incremental Update.
    /// This appends modified objects to the end of the file, followed by a new XREF table and trailer.
    /// This preserves original file data and digital signatures.
    pub fn save_incremental<P: AsRef<Path>>(&mut self, path: P, modified_objects: Vec<(ObjectId, crate::object::PdfObject)>) -> Result<(), PdfError> {
        let mut out_file = OpenOptions::new().write(true).append(true).open(path)?;

        let original_file_size = out_file.seek(SeekFrom::End(0))?;

        // Ensure the file ends with a newline before appending
        out_file.write_all(b"\n")?;

        let mut current_offset = original_file_size + 1;
        let mut new_xref_entries = Vec::new();

        // Write modified objects
        for (id, obj) in modified_objects {
            // Track this new offset BEFORE writing the header
            new_xref_entries.push((id.object_number, id.generation_number, current_offset));

            // Write object header
            let header = format!("{} {} obj\n", id.object_number, id.generation_number);
            out_file.write_all(header.as_bytes())?;
            current_offset += header.len() as u64;

            // Write object body
            let body_bytes = serialize_object(&obj);
            out_file.write_all(&body_bytes)?;
            current_offset += body_bytes.len() as u64;

            // Write endobj
            let footer = b"\nendobj\n";
            out_file.write_all(footer)?;
            current_offset += footer.len() as u64;
        }

        let startxref = current_offset;

        // Write new XREF table
        out_file.write_all(b"xref\n")?;

        // In a real implementation, we should group adjacent object numbers into subsections.
        // For MVP, we write each object as its own subsection of size 1.
        for (obj_num, gen_num, offset) in &new_xref_entries {
            let subsection_header = format!("{} 1\n", obj_num);
            out_file.write_all(subsection_header.as_bytes())?;

            // XREF entry: 10 bytes offset, space, 5 bytes gen, space, 'n', space, CRLF = 20 bytes exact
            let entry = format!("{:010} {:05} n \r\n", offset, gen_num);
            out_file.write_all(entry.as_bytes())?;
        }

        // Write new trailer
        out_file.write_all(b"trailer\n")?;

        let mut max_obj_id = self.xref_table.entries.keys().max().copied().unwrap_or(0);
        for (id, _, _) in &new_xref_entries {
            if *id > max_obj_id { max_obj_id = *id; }
        }

        let prev_str = match self.original_startxref {
            Some(prev) => format!(" /Prev {} ", prev),
            None => "".to_string(),
        };

        let trailer_str = format!("<< /Size {}{}>>\n", max_obj_id + 1, prev_str);
        out_file.write_all(trailer_str.as_bytes())?;

        // Write startxref and EOF
        let eof_block = format!("startxref\n{}\n%%EOF\n", startxref);
        out_file.write_all(eof_block.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::object::PdfObject;
    use std::io::Read;

    #[test]
    fn test_incremental_save() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"%PDF-1.4\n1 0 obj\n<< /Old true >>\nendobj\nxref\n0 2\n0000000000 65535 f \n0000000009 00000 n \ntrailer\n<< /Size 2 >>\nstartxref\n34\n%%EOF").unwrap();

        let mut doc = PdfDocument::open(file.path()).unwrap();
        doc.original_startxref = Some(34);

        let mut modified = Vec::new();
        modified.push((
            ObjectId { object_number: 1, generation_number: 0 },
            PdfObject::Dictionary(crate::object::PdfDictionary::new()) // Replaced with empty dict
        ));

        // The save method appends to the file
        doc.save_incremental(file.path(), modified).unwrap();

        let mut f = std::fs::File::open(file.path()).unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();

        assert!(content.contains("1 0 obj"));
        assert!(content.contains("<<\n>>")); // Our new serialized empty dict
        assert!(content.contains("startxref"));
        assert!(content.contains("%%EOF"));
        assert!(content.contains("/Prev 34"));
    }
}
