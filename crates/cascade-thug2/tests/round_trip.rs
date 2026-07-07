#[tokio::test]
async fn round_trip() {
    let entries = cascade_thug2::find_entries(&cascade_test::entries_dir("thug2"));
    cascade_test::test_entries::<cascade_thug2::Save>(entries).await
}
