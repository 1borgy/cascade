#[tokio::test]
async fn round_trip() {
    let entries = cascade_thug::find_entries(&cascade_test::entries_dir("thug"));
    cascade_test::test_entries::<cascade_thug::Save, cascade_thug::Cas>(entries).await
}
