# TQS v2 Overview

`tqs` v2 is a filesystem-first task queue for terminal-heavy workflows.

Tasks are stored as Markdown files with YAML frontmatter. The system is designed to be fast to capture into, easy to edit with any Markdown editor, and straightforward to inspect or automate through the filesystem.

## Goals

- extremely fast task capture
- markdown-native storage
- queue-based workflow
- human-readable and scriptable files
- editor-agnostic operation
- strong fit for automation and LLM tooling

## Non-Goals

The first version deliberately excludes:

- recurring tasks
- calendar scheduling
- reminders or notifications
- task dependencies
- priority scoring systems
- server or API mode
- built-in sync
- heavyweight project management features

## Task Model

A task is one Markdown file containing:

- YAML frontmatter with metadata
- a freeform Markdown body for notes, context, and subtasks

Required metadata fields:

- `id`
- `title`
- `queue`
- `created_at`
- `updated_at`

Optional first-class fields:

- `tags`
- `source`
- `project`
- `completed_at`
- `daily_note`

## Queue Model

Tasks move through work-horizon queues:

- `inbox`
- `now`
- `next`
- `later`
- `done`

Queue semantics:

- `inbox`: fast capture, minimal information required
- `now`: small set of tasks for immediate focus
- `next`: short-term tasks expected soon
- `later`: backlog and medium-term work
- `done`: completed tasks archive

## Workflow

The core workflow is:

1. Capture quickly into `inbox`.
2. Review and move tasks into the right queue.
3. Work from `now`.
4. Complete tasks into `done`.

## Filesystem-First Positioning

`tqs` is defined in terms of configured directories, not a specific note-taking tool.

- `tasks_root` is the directory containing queue subdirectories.
- `daily_notes_dir` is an optional directory for generic Markdown daily notes.

Users who want to integrate with Obsidian can point those directories inside a vault, but that is a deployment choice rather than part of the core architecture.
