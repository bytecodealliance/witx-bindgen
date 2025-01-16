mod async_module;

async_module::export!(Guest with_types_in async_module);

struct Guest;

impl async_module::exports::test::test::string_delay::Guest for Guest {
    async fn forward(s: String) -> String {
        match s.as_str() {
            "A" => "directly returned".into(),
            "B" => {
                async_module::test::test::wait::sleep(5_000_000_000).await;
                "after five seconds".into()
            }
            _ => {
                async_module::test::test::wait::sleep(1_000_000_000).await;
                "after one second".into()
            }
        }
    }
}
