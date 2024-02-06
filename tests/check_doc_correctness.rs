use logix::config::Logix;
use logix_type::LogixLoader;
use logix_vfs::RelFs;

#[test]
fn check_readme() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::create_dir(dir.path().join("config")).unwrap();
    std::fs::write(dir.path().join("config/bashrc"), "export HELLO=world").unwrap();

    std::fs::create_dir(dir.path().join("ssh-keys")).unwrap();
    std::fs::write(dir.path().join("ssh-keys/github"), "sha2 xyz").unwrap();

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

    println!("Temp dir: {:?}", dir.path());
    for file in files {
        println!("Loading {file}");
        loader.load_file::<Logix>(&file).unwrap();
    }
}
