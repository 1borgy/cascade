#[tokio::test]
async fn round_trip() {
    let entries = cascade_thaw::find_entries(&cascade_test::entries_dir("thaw"));
    cascade_test::test_entries::<cascade_thaw::Save, cascade_thaw::Cas>(entries).await
}
