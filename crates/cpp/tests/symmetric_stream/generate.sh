#!/bin/sh
(cd stream/src ; ../../../../../../target/debug/wit-bindgen rust ../../wit/async_stream.wit --async all --symmetric)
