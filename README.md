# docquery-rs
docquery-rs is a Rust back-end that applies text embedding and generation models to produce valid answers to user queries about a PDF uploaded by the user.

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

## Citation

```bibtex
@inproceedings{becquin-2020-end,
    title = "End-to-end {NLP} Pipelines in Rust",
    author = "Becquin, Guillaume",
    booktitle = "Proceedings of Second Workshop for NLP Open Source Software (NLP-OSS)",
    year = "2020",
    publisher = "Association for Computational Linguistics",
    url = "https://www.aclweb.org/anthology/2020.nlposs-1.4",
    pages = "20--25",
}
```

## Acknowledgements

* Thank you to [Hugging Face](https://huggingface.co) for hosting a set of weights compatible with Rust-Bert.
* [bhaskatripathi/pdfGPT](https://github.com/bhaskatripathi/pdfGPT) for the chunking approach.
