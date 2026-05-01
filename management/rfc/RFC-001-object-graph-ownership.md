# RFC 001: PDF Object Graph Ownership in Rust

## Background
A PDF file is fundamentally a graph of objects (Dictionaries, Arrays, Streams, Numbers) connected by Indirect References (e.g., `10 0 R`).
Because objects can reference each other cyclically (e.g., a Page dictionary references its Parent Pages node, which references the Page), we have to design the object model carefully to satisfy Rust's strict borrow checker.

## Proposal
Instead of using `Rc<RefCell<PdfObject>>` everywhere (which hurts performance and prevents multi-threading), we should use an **Arena or Central Document Store** pattern.
1. Every parsed indirect object is owned by a `PdfDocument` struct in a `HashMap<ObjectId, PdfObject>`.
2. References between objects are stored strictly as `ObjectId` (an integer tuple `(obj_num, gen_num)`), NOT as Rust pointers or references.
3. When logic needs to traverse the graph, it asks the `PdfDocument` to resolve the `ObjectId`.

## Open Questions
- Should we cache parsed objects, or parse them lazily as requested?
- How do we handle incremental updates (saving edits without rewriting the whole file)?
