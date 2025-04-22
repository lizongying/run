# run

This is a terminal-based application written in Rust, designed to execute a series of tasks with interactive options.
The application displays a task list through a TUI (Text User Interface) and allows users to select and execute
corresponding operations. The application supports the following features:

1. Task Management: The app loads and displays a list of tasks (including direct tasks and tasks with options). Users
   can choose tasks and execute them.
2. Command Execution: Each task is associated with a command (e.g., shell commands) managed by the Shell structure.
3. Interactive Selection: For tasks with options, users can select different options, and the app will execute the
   corresponding command based on the selected option.
4. Terminal Interface: Tasks are displayed through a TUI interface, and the execution result is shown in real time.

The app supports:

- Multi-platform: It works on Linux, macOS, and Windows environments.
- Dynamic Updates: The output of task execution is dynamically updated and displayed in the terminal.

[漢](./README_HANT.md)

![image](./screenshots/run.webp)

## Configuration

- env: Shell address, such as `bash`, `/bin/bash`, `zsh`, `/bin/zsh`. By default, it’s empty. Use `echo $SHELL` to check.
- job: There are two types of jobs: one is for executing a single shell command, and the other allows selecting from multiple commands, which requires configuring options.

```yaml
#cat ~/.run.yml

env:
jobs:
  - label: Who am I
    cmd: who am i

  - label: Which
    default_option: 0
    options:
      - label: node
        cmd: which node

...
```

## Support for the project

![image](./screenshots/appreciate.png)