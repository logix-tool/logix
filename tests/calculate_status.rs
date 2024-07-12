use std::collections::HashMap;

use logix::{
    managed_file::{FileStatus, ManagedFile, Owner},
    Logix,
};

mod helper;

static ROOT_LOGIX: &str = r#"
Logix {
  home: UserProfile {
    username: "zeldor"
    name: "Zeldon Kingly"
    email: "zeldor@example.com"
    shell: Bash
    editor: "hx"
    packages: {
      helix: Custom {
        source: GitHub {
          owner: "helix-editor"
          repo: "helix"
        }
        config_dir: User {
          filter: Filter {
            ignore_starts_with: [
                "runtime/"
            ]
          }
        }
      }
    }
  }
}
"#;

struct WantStatus {
    modified: Vec<ManagedFile>,
    missing: Vec<ManagedFile>,
    local_added: Vec<ManagedFile>,
    logix_added: Vec<ManagedFile>,
    up_to_date: Vec<ManagedFile>,
}

impl WantStatus {
    fn assert_eq(&self, logix: &Logix) -> bool {
        let mut files = HashMap::new();

        {
            let Self {
                modified,
                missing,
                local_added,
                logix_added,
                up_to_date,
            } = self;

            for file in modified {
                assert_eq!(files.insert(file, FileStatus::Modified), None);
            }

            for file in missing {
                assert_eq!(files.insert(file, FileStatus::MissingFromBoth), None);
            }

            for file in local_added {
                assert_eq!(files.insert(file, FileStatus::LocalAdded), None);
            }

            for file in logix_added {
                assert_eq!(files.insert(file, FileStatus::LogixAdded), None);
            }

            for file in up_to_date {
                assert_eq!(files.insert(file, FileStatus::UpToDate), None);
            }
        }

        for (status, file) in logix.calculate_config_status().unwrap() {
            if let Some(want_status) = files.remove(&file) {
                if want_status != status {
                    panic!("Got unexpected status {status:?} for file {file:?} expected {want_status:?}");
                }
            } else {
                panic!("Got unexpected file {file:?} with status {status:?}");
            }
        }

        if let Some((file, status)) = files.iter().next() {
            panic!("Missing file {file:?} with status {status:?}");
        }

        true
    }
}

#[test]
fn no_files() {
    let fs = helper::TestFs::new(ROOT_LOGIX);
    let logix = fs.load_logix();
    let mut want = WantStatus {
        modified: vec![],
        missing: vec![fs.managed_logix_dotfile(Owner::Shell, ".bashrc")],
        local_added: vec![],
        logix_added: vec![],
        up_to_date: vec![],
    };

    assert!(want.assert_eq(&logix));

    // Add a dummy config file and make sure we notice it
    fs.write_config_file("helix/config.toml", "# Dummy config");
    want.local_added
        .push(fs.managed_logix_config("helix", "helix/config.toml"));
    assert!(want.assert_eq(&logix));

    // Add themes/custom.toml to the logix config and make sure we notice it
    fs.write_config_file("logix/config/helix/themes/custom.toml", "# Dummy theme");
    want.logix_added
        .push(fs.managed_logix_config("helix", "helix/themes/custom.toml"));
    assert!(want.assert_eq(&logix));

    // Add a modified version of themes/custom.toml to .config and make sure we notice it
    fs.write_config_file("helix/themes/custom.toml", "# Dummy theme 2");
    want.modified.extend(want.logix_added.pop());
    assert!(want.assert_eq(&logix));

    // Make themes/custom.toml identical
    fs.write_config_file("helix/themes/custom.toml", "# Dummy theme");
    want.up_to_date.extend(want.modified.pop());
    assert!(want.assert_eq(&logix));

    // Files in the runtime directory should be ignored
    fs.write_config_file("helix/runtime/whatever.txt", "# Dummy file");
    assert!(want.assert_eq(&logix));
}
