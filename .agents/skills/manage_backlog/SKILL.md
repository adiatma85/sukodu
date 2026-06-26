---
name: manage-backlog
description: Guidelines for managing the project backlog, tasks, and status tracking consistently using the Backlog.md format.
---

# Manage Backlog

This skill ensures that the project backlog, stored as local Markdown files in the `backlog/` directory (consistent with the [Backlog.md](https://github.com/MrLesk/Backlog.md) specification), is maintained in a consistent, structured, and accurate format.

## Guidelines

1. **Backlog Structure**:
   - `backlog/config.yml`: Project-wide settings (statuses, definition of done, etc.).
   - `backlog/tasks/`: Subdirectory containing individual task files.
   - Naming convention: `backlog/tasks/task-<id>-<title-in-kebab-case>.md`.

2. **Task File Format**:
   Each task file must have YAML frontmatter followed by markdown content (Description, Acceptance Criteria, etc.).
   
   Template:
   ```markdown
   ---
   id: task-<id>
   title: "<Task Title>"
   status: "To Do" | "In Progress" | "Done"
   assignee: []
   created_date: "YYYY-MM-DD"
   updated_date: "YYYY-MM-DD"
   labels:
     - "<label>"
   dependencies: []
   priority: "low" | "medium" | "high"
   ---

   ## Description
   <Detailed description of the task, its scope, and why it is needed>

   ## Acceptance Criteria
   - [ ] <Criterion 1>
   - [ ] <Criterion 2>

   ## Definition of Done
   - [ ] <Specific DOD condition if different from default>
   ```

3. **Task Progression & Status Updates**:
   - When a task is started, change its `status` in the frontmatter to `In Progress` and update `updated_date`.
   - When completed, change its `status` to `Done`, update `updated_date`, and check off the checkboxes in Acceptance Criteria and Definition of Done.
   - Run verification tests to ensure the implementation works before marking a task as Done.

4. **Consistency**:
   - Keep tasks independent, clear, and action-oriented.
   - Avoid manual alterations of system-level metadata keys.
