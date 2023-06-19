# docquery-rs
docquery-rs is a Rust back-end that applies text embedding and generation models to produce valid answers to user queries about a PDF uploaded by the user! Currently limited to PDFs with less than 200 pages.

## Tech

- Actix-web: A high-performance, actor-based web framework for Rust.
- lopdf: A Rust library for parsing and writing PDF documents.
- Rayon: A data parallelism library for Rust.

## TODO
- [x] Create a service to receive the PDF.
- [x] Chunk the text-content with the page number citation.
- [x] Set the chunks, embeddings in a store and return the key.
- [x] Create a service to get answers about the PDF.
- [ ] Cache responses.

## Credits
* [bhaskatripathi/pdfGPT](https://github.com/bhaskatripathi/pdfGPT) for the chunking approach.
