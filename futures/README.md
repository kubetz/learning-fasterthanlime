# Understanding Rust futures by going way too deep

You can find the article [here](https://fasterthanli.me/articles/understanding-rust-futures-by-going-way-too-deep).

This project is very similar to the final version from the article. The main differences are:
- update to (at that time) the newest dependnecies
  - `rustls` - TlsConnector creation is now wildy different
  - `tracing-subscriber` - requires `env-filter` feature now
- reading from the socket got modified to handle EOF gracefully
- fixed clippy findings, minor refactorings, extra comments