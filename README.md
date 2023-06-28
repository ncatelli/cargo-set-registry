# crate-edit-registry
A tool, based on and heavily pulling from [cargo-edit](https://github.com/killercup/cargo-edit), for modifying the registry on a set of dependencies in a workspace.

## Usage

```bash
Usage: crate-edit-registry [OPTIONS] <REGISTRY>

Arguments:
  <REGISTRY>  Registry to update dependency to

Options:
      --manifest-path <PATH>  Path to the manifest to upgrade
  -p, --package <PKGID>       Package id of the crate to change the registry of
      --dry-run               Print changes to be made without making them
      --exclude <EXCLUDE>     Exclude a crate from the modification
      --locked                Require `Cargo.toml` to be up to date
  -h, --help                  Print help
  -V, --version               Print version
```

```bash
crate-edit-registry -p <dependency to update> <registry id> --dry-run
```

### Example
Given the following dependency in n number of workspaces in a project.

```toml
[dependencies]
some_dep = { version = "0.1.0" }
```

Running a dry run modification to add an `internal` registry id would output the following without making changes:

```bash
crate-edit-registry -p 'some_dep' 'internal' --dry-run
    Updating some_dep's dependency to add registry "internal"
warning: aborting set-registry due to dry run
```

Finally the changes can be applied with:

```bash
crate-edit-registry -p 'some_dep' 'internal'
    Updating some_dep's dependency to add registry "internal"
```

Which will yield the following in place change to all sub crates

```toml
[dependencies]
some_dep = { version = "0.1.0" , registry = "internal" }
```
