# Logix - Your Config Management Companion

Welcome to Logix, a lightweight and versatile command-line tool designed to simplify and enhance configuration management. Inspired by the concept of home-manager, Logix offers an intuitive way to organize, maintain, and extend your system configurations effortlessly. Whether you're a seasoned sysadmin, a developer, or simply someone who wants to keep their setup in Git, Logix is here to make your life easier.

## Goals

- Deploying across multiple machines and distros should yield a similar result.
- Deploying a new config after updates should be safe and atomic where possible.
- Enable system configurations to be stored and version-controlled.

## Current Focus

Logix is in an experimental phase, primarily focused on meeting the needs of its developers. The emphasis is on adding new features, and `todo!()` will be heavily utilized. The current goal is for the tool to work reliably on the developers' systems. Because of this focus, the configuration schema may change frequently and not be fully implemented. Documentation will also be postponed until much later. Until we reach the stable release of version 1.0, the tool should be considered experimental. If anyone wants to become a developer and either add new features, test it out, or just fix missing code they are welcome.

# Example

**Warning:** this example is likely outdated

```logix
/*
This is a config file example
*/
Logix { // The root of a config is always Logix
  home: UserProfile {
    username: "zeldor"
    name: "Zeldon Kingly"
    email: "zeldor@example.com"
    shell: Bash
    editor: Helix
    // ssh config, using the Open SSH provider
    ssh: OpenSSH {
      // Use the systemd version of the agent
      agent: SystemD
      keys: {
        github: @include("ssh-keys/github")
      }
    }
  }
}
```

# License

This project is licensed under the Mozilla Public License 2.0 (MPL). All contributions to this project will be covered by the MPL, unless an exception is explicitly stated at the top of the contributed file. For more details, see the [LICENSE](LICENSE) file in the repository
