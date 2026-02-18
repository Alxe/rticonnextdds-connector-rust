# Threading

The underlying RTI Connector C API is not thread-safe. The Rust bindings provide
synchronization around the native connector.

## Practical guidance about threading

Keep each `Input` or `Output` on a single thread at a time. If you need to share
work, consider a worker thread that owns the handle and communicates via
channels with the rest of your application.

While the connector uses internal locks for native access, this is not a
guarantee of safe concurrent access to the same `Input` or `Output`. Treat the
API as single-threaded unless you control synchronization at the application
level.
