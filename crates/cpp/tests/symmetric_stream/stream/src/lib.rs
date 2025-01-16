use futures::{SinkExt, StreamExt};
use stream_world::test::test::stream_source::create;
use wit_bindgen_symmetric_rt::async_support;

mod stream_world;

stream_world::export!(MyStruct with_types_in stream_world);

struct MyStruct;

impl stream_world::exports::test::test::stream_test::Guest for MyStruct {
    async fn create() -> async_support::StreamReader<u32> {
        let (mut writer, reader) = async_support::stream_support::new_stream();
        let mut input = create().await;

        async_support::spawn(async move {
            while let Some(values) = input.next().await {
                println!("received {} values", values.len());
                for value in values {
                    writer.feed(vec![value, value + 1]).await.unwrap();
                }
            }
        });
        return reader;
    }
}
