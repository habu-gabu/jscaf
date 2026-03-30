# jscaf

A simple Rust CLI to scaffold Java classes, interfaces, enums, records, and exceptions. Intellij installation is somehow broken in my system so I made this to scaffold java classes.

## Usage

```bash
jscaf new --type <FILETYPE> --name <NAMESPACE>
````

### Arguments

* `--type, -t` : The type of Java file to create. Options:
  `class`, `interface`, `enum`, `record`, `checked`, `unchecked`
* `--name, -n` : Fully qualified class name, including package, e.g., `service.UserService`