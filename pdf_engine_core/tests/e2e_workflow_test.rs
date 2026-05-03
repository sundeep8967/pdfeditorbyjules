use std::io::Write;
use tempfile::NamedTempFile;

use pdf_engine_core::catalog::DocumentCatalog;
use pdf_engine_core::content::parse_page_contents;
use pdf_engine_core::document::PdfDocument;
use pdf_engine_core::edit::{replace_text_in_operations, serialize_content_operations};
use pdf_engine_core::graphics::GraphicsStateProcessor;
use pdf_engine_core::object::{ObjectId, PdfDictionary, PdfObject, PdfStream};
use pdf_engine_core::render::render_page_to_pixels;
use pdf_engine_core::xref::XrefEntry;

/// Simulates a complete user journey: Open PDF, Find Text, Modify Text, Render Preview, Save Optimized.
#[test]
fn test_full_sdk_lifecycle() {
    // 1. Setup a valid mock PDF with a content stream
    let mut file = NamedTempFile::new().unwrap();
    let pdf_data = b"%PDF-1.4\n\
1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [ 3 0 R ] >>\nendobj\n\
3 0 obj\n<< /Type /Page /Contents 4 0 R >>\nendobj\n\
4 0 obj\n<< /Length 45 >>\nstream\n\
BT\n\
/F1 12 Tf\n\
100 200 Td\n\
(Hello World) Tj\n\
ET\n\
endstream\nendobj\n\
xref\n0 5\n\
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000051 00000 n \n\
0000000087 00000 n \n\
0000000129 00000 n \n\
trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n199\n%%EOF";

    file.write_all(pdf_data).unwrap();
    file.flush().unwrap();

    // 2. Open Document
    let mut doc = PdfDocument::open(file.path()).expect("Failed to open document");

    // Manually mock the parsed XREF offsets to match exactly what is in the buffer above
    // In production, `open` or the first interaction would parse this via `parser::parse_xref_table`.
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
            byte_offset: 51,
            generation_number: 0,
        },
    );
    // Fix offset for object 3, '3 0 obj' is at index 88
    doc.xref_table.entries.insert(
        3,
        XrefEntry::InUse {
            byte_offset: 88,
            generation_number: 0,
        },
    );
    // Fix offset for object 4, '4 0 obj' is at index 129
    doc.xref_table.entries.insert(
        4,
        XrefEntry::InUse {
            byte_offset: 129,
            generation_number: 0,
        },
    );

    let mut trailer = PdfDictionary::new();
    trailer.insert(
        "Root",
        PdfObject::Reference(ObjectId {
            object_number: 1,
            generation_number: 0,
        }),
    );
    doc.xref_table.trailer_dict = Some(trailer);

    // 3. Traverse Catalog to find pages
    let _raw3 = doc
        .get_raw_object_bytes(ObjectId {
            object_number: 3,
            generation_number: 0,
        })
        .unwrap();
    // Dynamic offset detection for stable test without mock strings breaking
    let pdf_str = std::str::from_utf8(pdf_data).unwrap();
    let off1 = pdf_str.find("1 0 obj").unwrap() as u64;
    let off2 = pdf_str.find("2 0 obj").unwrap() as u64;
    let off3 = pdf_str.find("3 0 obj").unwrap() as u64;
    let off4 = pdf_str.find("4 0 obj").unwrap() as u64;
    doc.xref_table.entries.insert(
        1,
        XrefEntry::InUse {
            byte_offset: off1,
            generation_number: 0,
        },
    );
    doc.xref_table.entries.insert(
        2,
        XrefEntry::InUse {
            byte_offset: off2,
            generation_number: 0,
        },
    );
    doc.xref_table.entries.insert(
        3,
        XrefEntry::InUse {
            byte_offset: off3,
            generation_number: 0,
        },
    );
    doc.xref_table.entries.insert(
        4,
        XrefEntry::InUse {
            byte_offset: off4,
            generation_number: 0,
        },
    );
    let pages = DocumentCatalog::get_all_pages(&mut doc).expect("Failed to traverse pages");
    assert_eq!(pages.len(), 1);
    let page_id = pages[0];

    // 4. Extract Page Content Streams
    let mut ops = parse_page_contents(&mut doc, page_id).expect("Failed to parse contents");

    // 5. Build Graphics State and Extract Text Bounding Boxes
    let mut proc = GraphicsStateProcessor::new();
    let text_blocks = proc.extract_text(&ops).expect("Failed to extract text");

    assert_eq!(text_blocks.len(), 1);
    assert_eq!(text_blocks[0].text, "Hello World");

    // 6. Mutate the Text (Editing)
    let replacements = replace_text_in_operations(&mut ops, "Hello World", "PDFgear Rules");
    assert_eq!(replacements, 1);

    // 7. Re-serialize the modified content
    let new_stream_bytes = serialize_content_operations(&ops);
    assert!(String::from_utf8_lossy(&new_stream_bytes).contains("PDFgear Rules"));

    // 8. Render the modified page to pixels
    // Render expects ops, we pass our modified ops.
    let pixels = render_page_to_pixels(500, 500, &ops).expect("Failed to render page");
    assert_eq!(pixels.len(), 500 * 500 * 4); // RGBA

    // 9. Save Optimized (Garbage Collection)
    // To save it properly, we would inject the `new_stream_bytes` back into the AST.
    // For this E2E test, we just ensure the save_optimized process doesn't panic on the graph traversal.
    let out_file = NamedTempFile::new().unwrap();
    doc.save_optimized(out_file.path())
        .expect("Failed to save optimized PDF");
}
