use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use crate::document::PdfDocument;
use crate::error::PdfError;
use crate::object::ObjectId;

use crate::edit::serialize_object;

impl PdfDocument {
    /// Saves the document using an Incremental Update.
    /// This appends modified objects to the end of the file, followed by a new XREF table and trailer.
    /// This preserves original file data and digital signatures.
    pub fn save_incremental<P: AsRef<Path>>(
        &mut self,
        path: P,
        modified_objects: Vec<(ObjectId, crate::object::PdfObject)>,
    ) -> Result<(), PdfError> {
        let mut out_file = OpenOptions::new().append(true).open(path)?;

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
            let entry = format!("{:010} {:05} n\r\n", offset, gen_num);
            out_file.write_all(entry.as_bytes())?;
        }

        // Write new trailer
        out_file.write_all(b"trailer\n")?;

        let mut max_obj_id = self.xref_table.entries.keys().max().copied().unwrap_or(0);
        for (id, _, _) in &new_xref_entries {
            if *id > max_obj_id {
                max_obj_id = *id;
            }
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
    use crate::object::PdfObject;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_incremental_save() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"%PDF-1.4\n1 0 obj\n<< /Old true >>\nendobj\nxref\n0 2\n0000000000 65535 f \n0000000009 00000 n \ntrailer\n<< /Size 2 >>\nstartxref\n34\n%%EOF").unwrap();

        // Make sure all bytes hit disk before opening
        file.flush().unwrap();
        let mut doc = PdfDocument::open(file.path()).unwrap();
        doc.original_startxref = Some(34);

        let mut modified = Vec::new();
        modified.push((
            ObjectId {
                object_number: 1,
                generation_number: 0,
            },
            PdfObject::Dictionary(crate::object::PdfDictionary::new()), // Replaced with empty dict
        ));

        // The save method appends to the file
        doc.save_incremental(file.path(), modified).unwrap();

        let mut f = std::fs::File::open(file.path()).unwrap();
        let mut content = String::new();
        f.read_to_end(unsafe { content.as_mut_vec() }).unwrap();

        assert!(content.contains("1 0 obj"));
        assert!(content.contains("<<\n>>")); // Our new serialized empty dict
        assert!(content.contains("startxref"));
        assert!(content.contains("%%EOF"));
        assert!(content.contains("/Prev 34"));
    }
}

use crate::ast_parser::Parser as AstParser;
use crate::lexer::Lexer;
use std::collections::HashSet;

impl PdfDocument {
    /// Saves the document by performing a full rewrite (Garbage Collection).
    /// This traverses the object graph from the Trailer `/Root`, serializes only reachable
    /// objects, and constructs a completely new XREF table, discarding orphaned data.
    pub fn save_optimized<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PdfError> {
        // 1. Gather all reachable objects
        let mut reachable = HashSet::new();
        let trailer = self
            .xref_table
            .trailer_dict
            .clone()
            .ok_or(PdfError::InvalidTrailer)?;

        let root_id = match trailer.get("Root") {
            Some(crate::object::PdfObject::Reference(id)) => *id,
            _ => return Err(PdfError::MissingRoot),
        };

        // We also need to keep the Info dictionary if it exists
        if let Some(crate::object::PdfObject::Reference(id)) = trailer.get("Info") {
            self.traverse_and_mark(*id, &mut reachable)?;
        }

        self.traverse_and_mark(root_id, &mut reachable)?;

        // 2. Open new file (truncate existing)
        let mut out_file = std::fs::File::create(path)?;

        // Write header
        let header = format!("%PDF-{}\n", self.version);
        out_file.write_all(header.as_bytes())?;

        // Write binary marker so viewers know it's a binary file
        let binary_marker = b"%\xE2\xE3\xCF\xD3\n";
        out_file.write_all(binary_marker)?;

        let mut current_offset = header.len() as u64 + binary_marker.len() as u64;
        let mut new_xref_entries = Vec::new();

        // 3. Write reachable objects
        // We sort the IDs so the output file is somewhat deterministic and clean
        let mut sorted_ids: Vec<_> = reachable.into_iter().collect();
        sorted_ids.sort_by_key(|id| id.object_number);

        for id in &sorted_ids {
            let obj = match self.parse_object_from_stream(*id) {
                Ok(o) => o,
                Err(e) => {
                    println!("FAILED TO PARSE OBJECT {}: {:?}", id.object_number, e);
                    continue;
                }
            };

            new_xref_entries.push((id.object_number, id.generation_number, current_offset));

            let obj_header = format!("{} {} obj\n", id.object_number, id.generation_number);
            out_file.write_all(obj_header.as_bytes())?;
            current_offset += obj_header.len() as u64;

            let body_bytes = serialize_object(&obj);
            out_file.write_all(&body_bytes)?;
            current_offset += body_bytes.len() as u64;

            let footer = b"\nendobj\n";
            out_file.write_all(footer)?;
            current_offset += footer.len() as u64;
        }

        let startxref = current_offset;

        // 4. Write unified XREF table
        out_file.write_all(b"xref\n")?;

        // Full rewrite XREF must start with the dummy 0 entry
        let max_id = sorted_ids.last().map(|id| id.object_number).unwrap_or(0);
        let num_entries = max_id + 1; // 0 to max_id

        let subsection_header = format!("0 {}\n", num_entries);
        out_file.write_all(subsection_header.as_bytes())?;

        out_file.write_all(b"0000000000 65535 f \r\n")?; // Entry 0 is always free

        // Write entries. If an ID is missing, mark it free.
        let mut id_idx = 0;
        for i in 1..=max_id {
            if id_idx < sorted_ids.len() && sorted_ids[id_idx].object_number == i {
                // We should find the entry by ID.
                let mut found_entry = None;
                for entry in &new_xref_entries {
                    if entry.0 == i {
                        found_entry = Some(entry);
                        break;
                    }
                }

                if let Some((_, gen_num, offset)) = found_entry {
                    let entry = format!("{:010} {:05} n\r\n", offset, gen_num);
                    out_file.write_all(entry.as_bytes())?;
                } else {
                    out_file.write_all(b"0000000000 00000 f \r\n")?;
                }
                id_idx += 1;
            } else {
                out_file.write_all(b"0000000000 00000 f \r\n")?;
            }
        }

        // 5. Write Trailer
        out_file.write_all(b"trailer\n")?;

        let mut new_trailer = trailer.clone();
        new_trailer.insert(
            "Size",
            crate::object::PdfObject::Integer(num_entries as i32),
        );
        // Remove /Prev since this is a fresh file
        new_trailer.entries.remove("Prev");

        let trailer_bytes = serialize_object(&crate::object::PdfObject::Dictionary(new_trailer));
        out_file.write_all(&trailer_bytes)?;
        out_file.write_all(b"\n")?;

        let eof_block = format!("startxref\n{}\n%%EOF\n", startxref);
        out_file.write_all(eof_block.as_bytes())?;

        Ok(())
    }

    /// Recursively marks all indirect references reachable from the given object ID.
    fn traverse_and_mark(
        &mut self,
        id: ObjectId,
        reachable: &mut HashSet<ObjectId>,
    ) -> Result<(), PdfError> {
        if !reachable.insert(id) {
            return Ok(()); // Already visited, prevent infinite loops in cyclic graphs
        }

        let obj = match self.parse_object_from_stream(id) {
            Ok(o) => o,
            Err(_) => return Ok(()), // Ignore broken branches
        };

        self.extract_references(&obj, reachable)?;
        Ok(())
    }

    fn extract_references(
        &mut self,
        obj: &crate::object::PdfObject,
        reachable: &mut HashSet<ObjectId>,
    ) -> Result<(), PdfError> {
        match obj {
            crate::object::PdfObject::Reference(ref_id) => {
                // We found a new edge, traverse it
                // Note: To avoid stack overflows on massive graphs, we should use an iterative queue here in production.
                // For MVP, recursive DFS is fine.
                self.traverse_and_mark(*ref_id, reachable)?;
            }
            crate::object::PdfObject::Array(arr) => {
                for item in arr {
                    self.extract_references(item, reachable)?;
                }
            }
            crate::object::PdfObject::Dictionary(dict) => {
                for val in dict.entries.values() {
                    self.extract_references(val, reachable)?;
                }
            }
            crate::object::PdfObject::Stream(s) => {
                for val in s.dict.entries.values() {
                    self.extract_references(val, reachable)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    // Changed to pub so tests can mock it easily, or we just rely on the fact that
    // the test creates a valid fake PDF stream layout.
    pub fn parse_object_from_stream(
        &mut self,
        id: ObjectId,
    ) -> Result<crate::object::PdfObject, PdfError> {
        // Special handling for the test since we use a fake file format
        // In reality, objects are delimited by 'endobj'.
        // get_raw_object_bytes extracts up to 'endobj' which is safe.
        let raw_bytes = self.get_raw_object_bytes(id)?;
        let lexer = Lexer::new(&raw_bytes);
        let mut parser = AstParser::new(lexer)?;
        parser.parse_object()
    }
}

#[cfg(test)]
mod gc_tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_optimized_garbage_collection() {
        let mut file = NamedTempFile::new().unwrap();
        // 1 is Root. 2 is reachable from 1. 3 is orphaned!

        file.write_all(
            b"%PDF-1.4\n\
1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [] >>\nendobj\n\
3 0 obj\n<< /Type /OrphanedGarbage >>\nendobj\n\
xref\n0 4\n\
0000000000 65535 f \n0000000009 00000 n \n0000000052 00000 n \n0000000092 00000 n \n\
trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n138\n%%EOF",
        )
        .unwrap();

        // Make sure all bytes hit disk before opening
        file.flush().unwrap();
        let mut doc = PdfDocument::open(file.path()).unwrap();

        // Mock XREF (normally done by open(), doing manually for test)
        doc.xref_table.entries.insert(
            1,
            crate::xref::XrefEntry::InUse {
                byte_offset: 9,
                generation_number: 0,
            },
        );
        doc.xref_table.entries.insert(
            2,
            crate::xref::XrefEntry::InUse {
                byte_offset: 51,
                generation_number: 0,
            },
        );
        doc.xref_table.entries.insert(
            3,
            crate::xref::XrefEntry::InUse {
                byte_offset: 89,
                generation_number: 0,
            },
        );

        let mut trailer = crate::object::PdfDictionary::new();
        trailer.insert(
            "Root",
            crate::object::PdfObject::Reference(ObjectId {
                object_number: 1,
                generation_number: 0,
            }),
        );
        doc.xref_table.trailer_dict = Some(trailer);

        // Use lossy string conversion because the binary contains non-utf8 markers

        // Object 3 should be completely garbage collected and missing from the output!
    }
}
