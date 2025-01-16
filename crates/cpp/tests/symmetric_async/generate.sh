#!/bin/sh
(cd async_module/src ; ../../../../../../target/debug/wit-bindgen rust ../../wit/async_module.wit --async all --symmetric)
