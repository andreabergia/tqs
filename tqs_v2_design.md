# Superseded Draft

This draft has been superseded by the canonical v2 docs under `docs/v2/`.

Primary replacements:

- `docs/v2/overview.md`
- `docs/v2/cli-contract.md`
- `docs/v2/storage-and-layout.md`
- `docs/v2/implementation-plan.md`

# TQS v2 -- Local Markdown Task Queue System

## Background

`tqs` originally started as a small CLI tool for managing tasks stored
as Markdown files with YAML frontmatter. The design worked well for
simple task tracking, but it lacked a workflow that reflects how many
developers actually manage work during the day.

In particular, the current workflow needs to support:

-   Many **tiny interrupt-driven tasks** (messages, notifications, quick
    requests)
-   **Medium-sized tasks** requiring notes and context
-   A **terminal-first workflow**
-   **LLM-friendly storage**
-   Tight **Obsidian integration**, including editing tasks and updating
    daily notes

Rather than maintaining backward compatibility with the current `tqs`
behavior, the project will evolve into a **local-first work queue
system** optimized for markdown-native workflows inside an Obsidian
vault.

The goal is to create a system that is:

-   extremely fast to capture tasks
-   flexible enough to store detailed notes
-   readable and scriptable via the filesystem
-   easy to automate and integrate with LLM tooling
-   comfortable to browse and edit in Obsidian

------------------------------------------------------------------------

# Functionalities

## Core Principles

The system follows several guiding principles:

-   **Local-first**: all state lives in the filesystem.
-   **Markdown-native**: tasks are plain markdown files with YAML
    frontmatter.
-   **Terminal-first capture**: adding a task must be extremely fast.
-   **Obsidian-native editing**: tasks are easily editable inside an
    Obsidian vault.
-   **Queue-based workflow**: tasks move through work horizons.
-   **LLM accessibility**: tasks remain simple markdown documents that
    tools can parse.

------------------------------------------------------------------------

# Task Model

A task is represented as a **single Markdown file**.

Each task contains:

-   YAML frontmatter with metadata
-   a markdown body for notes, context, and subtasks

Example:

``` markdown
---
id: 20260309-103412-reply-aws-billing
title: Reply to AWS billing alert
queue: inbox
created_at: 2026-03-09T10:34:12+01:00
updated_at: 2026-03-09T10:34:12+01:00
tags: [aws, finance]
source: manual
project:
completed_at:
daily_note:
---

# Reply to AWS billing alert

## Context

Billing alert came from finance.

## Notes

Need to check Cost Explorer.
```

## Required metadata fields

Required fields:

-   `id`
-   `title`
-   `queue`
-   `created_at`
-   `updated_at`

Optional but first-class fields:

-   `tags`
-   `source`
-   `project`
-   `completed_at`
-   `daily_note`

------------------------------------------------------------------------

# Queue System

Tasks move through **work horizon queues**.

Initial queues:

-   `inbox`
-   `now`
-   `next`
-   `later`
-   `done`

Each queue corresponds to a folder in the filesystem.

Example layout inside an Obsidian vault:

    Vault/
      Tasks/
        inbox/
        now/
        next/
        later/
        done/
      Daily/

## Queue semantics

### Inbox

Quick capture location for new tasks.

Characteristics:

-   minimal information required
-   high volume allowed
-   processed later

### Now

Tasks intended for **immediate work** (roughly the next couple of
hours).

Characteristics:

-   intentionally small
-   represents active focus

### Next

Short-term work to be done in the near future (typically within the
week).

### Later

Medium- and long-term tasks or backlog items.

### Done

Completed tasks.

When a task is moved to `done`, metadata is updated and optional daily
note integration is triggered.

------------------------------------------------------------------------

# Core Commands

The CLI should support a small set of focused commands.

## Add

Fast capture of a task.

Example:

    tqs add "reply to AWS billing alert"

Behavior:

-   create new task
-   place in `inbox`
-   use minimal template

## List

Display tasks by queue.

Examples:

    tqs list
    tqs list now
    tqs list inbox

## Move

Move a task between queues.

Examples:

    tqs move <task> now
    tqs move <task> next

## Done

Complete a task.

Behavior:

-   move task to `done`
-   set `completed_at`
-   optionally update daily note

Example:

    tqs done <task>

## Edit

Open task in the configured editor.

Example:

    tqs edit <task>

## Show

Display task content in the terminal.

Example:

    tqs show <task>

## Find

Search tasks by text or metadata.

Examples:

    tqs find aws
    tqs find --tag finance

------------------------------------------------------------------------

# Obsidian Integration

Tasks are stored **inside an Obsidian vault** so they can be edited and
navigated directly.

Key integration features:

### Vault Storage

Tasks reside inside:

    <Vault>/Tasks/

### Editing

Tasks can be opened in an external editor or directly edited inside
Obsidian.

### Daily Note Integration

When a task is completed, `tqs` can append an entry to the current daily
note.

Example entry:

    ## Completed Tasks

    - [x] [[Tasks/done/20260309-103412-reply-aws-billing|Reply to AWS billing alert]]

This allows daily notes to serve as a **work journal** while the task
file remains the canonical record.

------------------------------------------------------------------------

# Configuration

Configuration should be minimal.

Example:

``` toml
vault_path = "/Users/user/Obsidian/MyVault"
tasks_dir = "Tasks"
daily_notes_dir = "Daily"

daily_note_date_format = "%Y-%m-%d"
append_completed_to_daily_note = true

daily_note_completed_section = "Completed Tasks"
```

------------------------------------------------------------------------

# Non-Goals for v1

The following features are explicitly **out of scope for the first
iteration**:

-   recurring tasks
-   due-date scheduling systems
-   task dependencies
-   reminders or notifications
-   multi-device synchronization logic
-   server or API mode
-   calendar integration

------------------------------------------------------------------------

# High-Level Implementation Plan

The existing `tqs` codebase will serve as a **starting point**, but
backward compatibility with the previous behavior is not required.

The project will evolve by:

1.  **Reusing useful components**
2.  **Replacing the old domain model**
3.  **Adapting storage to the new queue-based structure**

## Step 1 -- Identify reusable components

Parts of the existing code that are likely reusable include:

-   Markdown parsing and serialization
-   YAML frontmatter handling
-   editor integration
-   CLI scaffolding
-   filesystem utilities
-   interactive task selection

## Step 2 -- Replace the task domain model

The current task lifecycle (`open` / `closed`) will be replaced by the
**queue-based workflow**.

This includes:

-   new metadata schema
-   new queue semantics
-   queue-based filesystem layout

## Step 3 -- Implement queue-based storage

Tasks will be stored as:

    <Vault>/Tasks/<queue>/<id>.md

Moving a task between queues corresponds to moving the file between
folders.

## Step 4 -- Implement the new CLI surface

New commands such as:

-   `add`
-   `list`
-   `move`
-   `done`
-   `edit`
-   `find`

will become the primary interface.

## Step 5 -- Implement Obsidian integration

Add support for:

-   locating the vault
-   locating daily notes
-   appending completion entries
-   creating missing sections if needed

## Step 6 -- Incremental refinement

Once the core workflow is stable, further improvements can be explored:

-   improved search
-   interactive inbox processing
-   project views
-   LLM integrations

------------------------------------------------------------------------

# Summary

The new `tqs` will become a **local markdown-native work queue system**
designed for:

-   terminal-heavy workflows
-   Obsidian integration
-   tasks that may contain detailed notes
-   simple filesystem-based storage
-   future automation and LLM tooling

By evolving the existing codebase rather than starting from scratch,
development can focus on the **new workflow model** while preserving
useful infrastructure already implemented in the project.
