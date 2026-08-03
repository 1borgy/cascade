#[tokio::test]
async fn round_trip() {
    let entries = cascade_thps3::find_entries(&cascade_test::entries_dir("thps3"));
    cascade_test::test_entries::<cascade_thps3::Save, cascade_thps3::Cas>(entries).await
}
