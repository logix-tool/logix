use logix::Logix;
use logix_type::LogixLoader;
use logix_vfs::RelFs;

#[test]
fn check_readme() {
    let dir = tempfile::tempdir().unwrap();

    let mut files = Vec::new();

    for (i, cur) in include_str!("../README.md")
        .split("```logix")
        .skip(1)
        .enumerate()
    {
        let name = format!("file_{i}.logix");

        let (source, _) = cur.split_once("```").unwrap_or_else(|| panic!("{cur:?}"));
        assert!(&source.contains("Logix {"), "{source:?}");

        std::fs::write(dir.path().join(&name), source.as_bytes()).unwrap();

        files.push(name);
    }

    assert_ne!(files.len(), 0);

    let mut loader = LogixLoader::new(RelFs::new(dir.path()));

    for file in files {
        loader.load_file::<Logix>(&file).unwrap();
    }
}
