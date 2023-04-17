# docgpt-rs
docgpt-rs is a Rust backend for a mobile app that applies text similarity and generation models to split PDFs into vectors of page-numbered strings, and produces valid answers to user questions.

## Tech

- Actix-web: A high-performance, actor-based web framework for Rust.
- lopdf: A Rust library for parsing and writing PDF documents.

## TODO
- [x] Create a service to receive the PDF
- [x] Chunk the text-content with the page number citation 
- [x] Set the chunks to a Redis store and return the key
- [ ] Create a service to get answers about the PDF

## Credits
* [bhaskatripathi/pdfGPT](https://github.com/bhaskatripathi/pdfGPT) for the chunking approach.
