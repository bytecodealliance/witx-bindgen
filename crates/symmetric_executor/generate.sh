#!/bin/sh
(cd rust-client/src;../../../../target/debug/wit-bindgen rust ../../wit -w module --symmetric --async none)
(cd src;../../../target/debug/wit-bindgen rust ../wit -w executor --symmetric --async none)
(cd symmetric_stream/src;../../../../target/debug/wit-bindgen rust ../../wit -w stream-impl --symmetric --async none)
