use logix::{status::Status, ManagedFile};

mod helper;

static ROOT_LOGIX: &str = r#"
Logix {
  home: UserProfile {
    username: "zeldor"
    name: "Zeldon Kingly"
    email: "zeldor@example.com"
    shell: Bash
    editor: Helix
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
    fn assert_eq(&self, status: &Status) -> bool {
        if !helper::compare_iters(
            "Modified entries",
            self.modified.iter(),
            status.modified().iter(),
        ) {
            return false;
        }

        if !helper::compare_iters(
            "Missing entries",
            self.missing.iter(),
            status.missing().iter(),
        ) {
            return false;
        }

        if !helper::compare_iters(
            "Local added entries",
            self.local_added.iter(),
            status.local_added().iter(),
        ) {
            return false;
        }

        if !helper::compare_iters(
            "Logix added entries",
            self.logix_added.iter(),
            status.logix_added().iter(),
        ) {
            return false;
        }

        if !helper::compare_iters(
            "Up to date entries",
            self.up_to_date.iter(),
            status.up_to_date().iter(),
        ) {
            return false;
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
        missing: vec![fs.managed_logix_dotfile(".bashrc")],
        local_added: vec![],
        logix_added: vec![],
        up_to_date: vec![],
    };

    assert!(want.assert_eq(&logix.calculate_status().unwrap()));

    // Add a dummy config file and make sure we notice it
    fs.write_config_file("helix/config.toml", "# Dummy config");
    want.local_added
        .push(fs.managed_logix_config("helix/config.toml"));
    assert!(want.assert_eq(&logix.calculate_status().unwrap()));

    // Add themes/custom.toml to the logix config and make sure we notice it
    fs.write_config_file("logix/config/helix/themes/custom.toml", "# Dummy theme");
    want.logix_added
        .push(fs.managed_logix_config("helix/themes/custom.toml"));
    assert!(want.assert_eq(&logix.calculate_status().unwrap()));

    // Add a modified version of themes/custom.toml to .config and make sure we notice it
    fs.write_config_file("helix/themes/custom.toml", "# Dummy theme 2");
    want.modified.extend(want.logix_added.pop());
    assert!(want.assert_eq(&logix.calculate_status().unwrap()));

    // Make themes/custom.toml identical
    fs.write_config_file("helix/themes/custom.toml", "# Dummy theme");
    want.up_to_date.extend(want.modified.pop());
    assert!(want.assert_eq(&logix.calculate_status().unwrap()));

    // Files in the runtime directory should be ignored
    fs.write_config_file("helix/runtime/whatever.txt", "# Dummy file");
    assert!(want.assert_eq(&logix.calculate_status().unwrap()));
}
