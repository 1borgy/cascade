#[tokio::test]
async fn round_trip() {
    let entries = cascade_thps4::find_entries(&cascade_test::entries_dir("thps4"));
    cascade_test::test_entries::<cascade_thps4::Save, cascade_thps4::Cas>(entries).await
}
