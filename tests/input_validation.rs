use pontis::fs_scan::build_diff_files;

mod support;

#[test]
fn file_dir_mixed_input_is_rejected() {
    let root = support::unique_temp_dir("pontis-mixed-input");
    let left_file = root.join("left.txt");
    let right_dir = root.join("right_dir");
    let left_dir = root.join("left_dir");
    let right_file = root.join("right.txt");

    std::fs::create_dir_all(&right_dir).expect("create right dir");
    std::fs::create_dir_all(&left_dir).expect("create left dir");
    std::fs::write(&left_file, "x\n").expect("create left file");
    std::fs::write(&right_file, "x\n").expect("create right file");

    let err1 = build_diff_files(left_file.as_path(), right_dir.as_path())
        .expect_err("mixed file/dir must fail");
    assert!(
        err1.to_string().contains("mixed input"),
        "unexpected error: {err1:#}"
    );

    let err2 = build_diff_files(left_dir.as_path(), right_file.as_path())
        .expect_err("mixed dir/file must fail");
    assert!(
        err2.to_string().contains("mixed input"),
        "unexpected error: {err2:#}"
    );

    std::fs::remove_dir_all(&root).expect("cleanup");
}
