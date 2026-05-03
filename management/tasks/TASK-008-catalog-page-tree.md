# TASK-008: Parse Document Catalog and Page Tree

## Status
[ ] Not Started | [ ] In Progress | [ ] In Review | [x] Done

## Owner
@jules

## Objective
The engine needs to find the entry point of the PDF (the Document Catalog via the `/Root` key in the trailer) and traverse the `/Pages` tree to assemble a list of all pages in the document.

## Acceptance Criteria
- [ ] Parse the `trailer` dictionary when parsing the XREF table.
- [ ] Implement a method to extract the `/Root` indirect reference.
- [ ] Traverse the `/Pages` tree, resolving `/Kids` arrays to flatten the tree into a list of `/Page` references.
- [ ] Unit tests for finding the trailer and traversing a mock page tree.
