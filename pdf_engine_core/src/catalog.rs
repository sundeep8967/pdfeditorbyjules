use crate::ast_parser::Parser as AstParser;
use crate::document::PdfDocument;
use crate::error::PdfError;
use crate::lexer::Lexer;
use crate::object::{ObjectId, PdfObject};

pub struct DocumentCatalog {
    pub root_id: ObjectId,
}

impl DocumentCatalog {
    /// Discovers the Document Catalog from the trailer and returns all Page references.
    pub fn get_all_pages(doc: &mut PdfDocument) -> Result<Vec<ObjectId>, PdfError> {
        let trailer = doc.xref_table.trailer().ok_or(PdfError::InvalidTrailer)?;

        let root_id = match trailer.get("Root") {
            Some(PdfObject::Reference(id)) => *id,
            _ => return Err(PdfError::MissingRoot),
        };

        // Load the catalog object
        let catalog_obj = load_object(doc, root_id)?;
        let catalog_dict = match catalog_obj {
            PdfObject::Dictionary(d) => d,
            _ => return Err(PdfError::InvalidPageTree),
        };

        let pages_root_id = match catalog_dict.get("Pages") {
            Some(PdfObject::Reference(id)) => *id,
            _ => return Err(PdfError::InvalidPageTree),
        };

        let mut page_list = Vec::new();
        traverse_page_tree(doc, pages_root_id, &mut page_list)?;

        Ok(page_list)
    }
}

/// Helper to extract, lex, and parse an object from the file stream.
fn load_object(doc: &mut PdfDocument, id: ObjectId) -> Result<PdfObject, PdfError> {
    let raw_bytes = doc.get_raw_object_bytes(id)?;
    let lexer = Lexer::new(&raw_bytes);
    let mut parser = AstParser::new(lexer)?;
    parser.parse_object()
}

/// Recursively traverses the Pages tree to flatten all "Page" nodes into a list.
fn traverse_page_tree(
    doc: &mut PdfDocument,
    node_id: ObjectId,
    out_pages: &mut Vec<ObjectId>,
) -> Result<(), PdfError> {
    let node_obj = load_object(doc, node_id)?;
    let node_dict = match node_obj {
        PdfObject::Dictionary(d) => d,
        _ => return Err(PdfError::InvalidPageTree),
    };

    let node_type = match node_dict.get("Type") {
        Some(PdfObject::Name(n)) => n.as_str(),
        _ => return Err(PdfError::InvalidPageTree),
    };

    if node_type == "Page" {
        out_pages.push(node_id);
        return Ok(());
    } else if node_type != "Pages" {
        return Err(PdfError::InvalidPageTree);
    }

    let kids_array = match node_dict.get("Kids") {
        Some(PdfObject::Array(arr)) => arr,
        _ => return Err(PdfError::InvalidPageTree),
    };

    for kid in kids_array {
        if let PdfObject::Reference(kid_id) = kid {
            // Recursive call limits: in a production SDK, we'd want to guard against infinite cycles here
            traverse_page_tree(doc, *kid_id, out_pages)?;
        } else {
            return Err(PdfError::InvalidPageTree);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xref::XrefEntry;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_traverse_page_tree() {
        // Construct a mock PDF where:
        // 1 0 obj: Catalog
        // 2 0 obj: Pages root
        // 3 0 obj: Page 1
        // 4 0 obj: Page 2

        let pdf_data = b"%PDF-1.4\n\
1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [ 3 0 R 4 0 R ] >>\nendobj\n\
3 0 obj\n<< /Type /Page >>\nendobj\n\
4 0 obj\n<< /Type /Page >>\nendobj\n";

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(pdf_data).unwrap();

        let mut doc = PdfDocument::open(file.path()).unwrap();

        // Mock the XREF table and Trailer
        doc.xref_table.entries.insert(
            1,
            XrefEntry::InUse {
                byte_offset: 9,
                generation_number: 0,
            },
        );
        doc.xref_table.entries.insert(
            2,
            XrefEntry::InUse {
                byte_offset: 58,
                generation_number: 0,
            },
        );
        doc.xref_table.entries.insert(
            3,
            XrefEntry::InUse {
                byte_offset: 114,
                generation_number: 0,
            },
        );
        doc.xref_table.entries.insert(
            4,
            XrefEntry::InUse {
                byte_offset: 147,
                generation_number: 0,
            },
        );

        let mut trailer = crate::object::PdfDictionary::new();
        trailer.insert(
            "Root",
            PdfObject::Reference(ObjectId {
                object_number: 1,
                generation_number: 0,
            }),
        );
        doc.xref_table.trailer_dict = Some(trailer);

        let pages = DocumentCatalog::get_all_pages(&mut doc).unwrap();

        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].object_number, 3);
        assert_eq!(pages[1].object_number, 4);
    }
}
