# Superseded Draft

This draft has been superseded by the canonical v2 docs under `docs/v2/`.

Primary replacements:

- `docs/v2/overview.md`
- `docs/v2/cli-contract.md`
- `docs/v2/storage-and-layout.md`
- `docs/v2/implementation-plan.md`

# TQS v2 Design Document

## Overview

`tqs` is evolving from a simple CLI task tracker into a **local-first,
markdown-native task queue system** designed for developer workflows.

The primary goal is to support a work style characterized by:

-   frequent **interrupt-driven tasks** (messages, notifications, quick
    actions)
-   **medium-sized work items** that require notes and context
-   heavy **terminal usage**
-   seamless **Obsidian integration**
-   **LLM-friendly storage**

Rather than maintaining backward compatibility with the current `tqs`
behavior, the project will evolve to support a **queue-based workflow**
and stronger integration with an Obsidian vault.

The resulting system should feel like a lightweight **local work
operating system** built entirely on markdown files.

------------------------------------------------------------------------

# Goals

## Primary goals

-   extremely fast **task capture**
-   **markdown-native storage**
-   seamless **editing inside Obsidian**
-   **queue-based workflow** reflecting real work horizons
-   **scriptable filesystem storage**
-   strong **LLM accessibility**

## Secondary goals

-   support storing **rich notes per task**
-   enable **terminal-first workflows**
-   allow simple **automation and integrations**
-   maintain **human-readable storage**

------------------------------------------------------------------------

# Non-Goals (v1)

The first version deliberately excludes the following:

-   recurring tasks
-   calendar scheduling
-   reminders or notifications
-   task dependencies
-   priority scoring systems
-   server/API mode
-   built-in multi-device sync
-   project management features beyond basic tagging

The focus of v1 is **workflow simplicity and speed**.

------------------------------------------------------------------------

# Core Concepts

## Tasks

A task is represented as **one markdown file** containing:

-   YAML frontmatter metadata
-   a markdown body for notes

Tasks may range from extremely small items (single-line tasks) to rich
documents with context and subtasks.

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

### Required fields

-   `id`
-   `title`
-   `queue`
-   `created_at`
-   `updated_at`

### Optional fields

-   `tags`
-   `source`
-   `project`
-   `completed_at`
-   `daily_note`

The body of the task remains **fully freeform markdown**.

------------------------------------------------------------------------

# Queue Model

Tasks move through **queues representing work horizons**.

Initial queues:

-   `inbox`
-   `now`
-   `next`
-   `later`
-   `done`

Queues correspond to folders in the filesystem.

Example vault structure:

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

Entry point for new tasks.

Characteristics:

-   minimal information required
-   high volume allowed
-   intended for later processing

### Now

Tasks for **immediate focus**.

Characteristics:

-   intentionally small set
-   represents current working context

### Next

Short-term tasks expected to be completed soon (typically this week).

### Later

Backlog of future work and medium-term tasks.

### Done

Completed tasks.

Completion triggers metadata updates and optionally a daily note entry.

------------------------------------------------------------------------

# CLI Commands

The CLI exposes a small set of focused commands.

## Add

Fast task capture.

    tqs add "reply to AWS billing alert"

Behavior:

-   creates task
-   places task in `inbox`
-   applies minimal template

------------------------------------------------------------------------

## List

Display tasks.

Examples:

    tqs list
    tqs list now
    tqs list inbox

The default view may show a compact dashboard including:

-   `now`
-   `inbox`
-   queue counts

------------------------------------------------------------------------

## Move

Move tasks between queues.

    tqs move <task> now
    tqs move <task> next

Moving a task corresponds to **moving the underlying file**.

------------------------------------------------------------------------

## Done

Complete a task.

    tqs done <task>

Behavior:

-   move file to `done`
-   set `completed_at`
-   update daily note (optional)

------------------------------------------------------------------------

## Edit

Open a task in the configured editor.

    tqs edit <task>

------------------------------------------------------------------------

## Show

Display task details in the terminal.

    tqs show <task>

------------------------------------------------------------------------

## Find

Search tasks.

Examples:

    tqs find aws
    tqs find --tag finance

Search should operate on:

-   titles
-   metadata
-   full markdown body

------------------------------------------------------------------------

# Obsidian Integration

`tqs` is designed to operate **inside an Obsidian vault**.

## Task Storage

Tasks are stored under:

    <Vault>/Tasks/

This allows:

-   browsing tasks in Obsidian
-   linking tasks to notes
-   editing tasks with full markdown features

------------------------------------------------------------------------

## Daily Note Integration

When a task is completed, `tqs` may append an entry to the daily note.

Example:

    ## Completed Tasks

    - [x] [[Tasks/done/20260309-103412-reply-aws-billing|Reply to AWS billing alert]]

This turns the daily note into a **work journal** while the task file
remains the canonical record.

------------------------------------------------------------------------

# Configuration

Configuration is intentionally minimal.

Example configuration:

``` toml
vault_path = "/Users/user/Obsidian/MyVault"
tasks_dir = "Tasks"
daily_notes_dir = "Daily"

daily_note_date_format = "%Y-%m-%d"
append_completed_to_daily_note = true
daily_note_completed_section = "Completed Tasks"
```

Key configuration fields:

  Field                            Purpose
  -------------------------------- -------------------------------
  vault_path                       path to Obsidian vault
  tasks_dir                        location of task folders
  daily_notes_dir                  location of daily notes
  append_completed_to_daily_note   enable daily note integration

------------------------------------------------------------------------

# High-Level Implementation Plan

The current `tqs` codebase will be **evolved rather than preserved**.

Backward compatibility is **not required**.

Development will focus on reshaping the tool around the new workflow
model.

## Step 1 --- Identify reusable components

Reuse existing infrastructure where possible:

-   markdown parsing
-   YAML frontmatter handling
-   editor integration
-   CLI scaffolding
-   filesystem helpers
-   interactive task selection

These pieces reduce implementation effort without constraining the new
design.

------------------------------------------------------------------------

## Step 2 --- Replace the task domain model

The current lifecycle (`open` / `closed`) will be removed.

It will be replaced with the **queue-based workflow**:

    inbox → now → next → later → done

This requires:

-   new metadata schema
-   queue semantics
-   queue-based storage layout

------------------------------------------------------------------------

## Step 3 --- Implement queue-based storage

Tasks will live at:

    <Vault>/Tasks/<queue>/<id>.md

Moving tasks between queues will simply move files between folders.

------------------------------------------------------------------------

## Step 4 --- Implement new CLI commands

Introduce the new command surface:

-   `add`
-   `list`
-   `move`
-   `done`
-   `edit`
-   `show`
-   `find`

Old command semantics can be removed if necessary.

------------------------------------------------------------------------

## Step 5 --- Implement Obsidian integration

Add support for:

-   vault detection
-   daily note resolution
-   safe daily note updates
-   automatic section creation

------------------------------------------------------------------------

## Step 6 --- Iterate on workflow improvements

Once the basic system works well, additional improvements may include:

-   interactive inbox processing
-   better search
-   project filtering
-   LLM integrations
-   task dashboards

These improvements should remain consistent with the **markdown-first
architecture**.

------------------------------------------------------------------------

# Summary

`tqs` v2 becomes a **local-first markdown task queue system** designed
for:

-   terminal-heavy workflows
-   Obsidian-based knowledge management
-   tasks that contain notes and context
-   simple filesystem-based storage
-   future automation and AI tooling

By evolving the existing codebase and focusing on the new queue
workflow, `tqs` can become a powerful yet lightweight productivity tool
that integrates naturally with both the terminal and Obsidian.
