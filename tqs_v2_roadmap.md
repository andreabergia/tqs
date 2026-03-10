# Superseded Draft

This draft has been superseded by the canonical v2 docs under `docs/v2/`.

Primary replacements:

- `docs/v2/overview.md`
- `docs/v2/cli-contract.md`
- `docs/v2/storage-and-layout.md`
- `docs/v2/implementation-plan.md`

# TQS v2 Implementation Roadmap

This document describes a **practical implementation roadmap** for
evolving the current `tqs` codebase into the new queue‑based system
described in the design document.

The goal is to avoid a risky "big rewrite" and instead evolve the
project through **small, well‑scoped steps** that keep the codebase
working at all times.

Each step should ideally correspond to one or a small number of pull
requests.

------------------------------------------------------------------------

# Guiding Principles

Implementation should follow these principles:

1.  **Reuse infrastructure where possible**
2.  **Replace the task domain model early**
3.  **Keep the CLI usable throughout the transition**
4.  **Prefer incremental migration over large rewrites**
5.  **Ensure filesystem behavior stays predictable**

------------------------------------------------------------------------

# Phase 1 --- Repository Preparation

## Step 1: Create the new documentation structure

Add documentation describing the new direction.

Suggested structure:

    docs/
      design.md
      roadmap.md

Actions:

-   commit the design document
-   commit this roadmap
-   update the README to mention the upcoming v2 evolution

Purpose:

-   clarify project direction before major code changes

------------------------------------------------------------------------

## Step 2: Introduce new queue concepts

Introduce the new queue types in the codebase:

    Inbox
    Now
    Next
    Later
    Done

Actions:

-   add a `Queue` enum
-   implement parsing from string
-   add serialization support

At this stage the enum can coexist with the old status system.

------------------------------------------------------------------------

# Phase 2 --- New Task Model

## Step 3: Define the new task schema

Introduce a new internal `Task` model matching the design document.

Fields should include:

-   id
-   title
-   queue
-   created_at
-   updated_at
-   tags (optional)
-   source (optional)
-   project (optional)
-   completed_at (optional)

Actions:

-   define new struct
-   implement YAML serialization/deserialization
-   support loading from markdown files

The markdown body should remain **fully freeform**.

------------------------------------------------------------------------

## Step 4: Implement queue‑based storage layout

Add support for the new filesystem layout:

    <Vault>/Tasks/<queue>/<id>.md

Actions:

-   implement directory resolution
-   implement queue directory creation
-   implement task file path resolution

At this stage tasks may still be loaded from the previous layout if
necessary.

------------------------------------------------------------------------

# Phase 3 --- New Core Operations

## Step 5: Implement task creation (`add`)

Introduce the new task creation workflow.

Example:

    tqs add "reply to AWS billing alert"

Behavior:

-   generate task id
-   create markdown file
-   place task in `inbox`
-   populate minimal template

Infrastructure reused:

-   markdown writing
-   editor integration

------------------------------------------------------------------------

## Step 6: Implement task listing

Introduce queue‑aware listing.

Examples:

    tqs list
    tqs list now
    tqs list inbox

Behavior:

-   enumerate queue directories
-   load metadata
-   display compact summary

Optional improvements:

-   queue counts
-   grouped display

------------------------------------------------------------------------

## Step 7: Implement task movement

Implement moving tasks between queues.

Example:

    tqs move <task> now

Behavior:

-   resolve task identifier
-   move file to queue folder
-   update metadata
-   update timestamps

This is a core operation in the new workflow.

------------------------------------------------------------------------

# Phase 4 --- Completion Workflow

## Step 8: Implement task completion (`done`)

Add task completion logic.

Example:

    tqs done <task>

Behavior:

-   move task to `done`
-   set `completed_at`
-   update metadata

This step introduces the first lifecycle operation in the new model.

------------------------------------------------------------------------

# Phase 5 --- Obsidian Integration

## Step 9: Implement vault configuration

Add configuration support for:

-   `vault_path`
-   `tasks_dir`
-   `daily_notes_dir`

Configuration sources may include:

-   config file
-   environment variables
-   CLI overrides

------------------------------------------------------------------------

## Step 10: Implement daily note integration

When a task is completed:

1.  locate today's daily note
2.  create it if necessary
3.  ensure the completion section exists
4.  append the task link

Example output:

    ## Completed Tasks

    - [x] [[Tasks/done/<task-id>|<task-title>]]

Edge cases to handle:

-   duplicate entries
-   missing directories
-   malformed notes

------------------------------------------------------------------------

# Phase 6 --- Search and Editing

## Step 11: Implement task editing

Reuse the existing editor integration.

Example:

    tqs edit <task>

Behavior:

-   resolve task file
-   open using configured editor

------------------------------------------------------------------------

## Step 12: Implement search

Add search functionality.

Example:

    tqs find aws

Search should operate across:

-   titles
-   metadata
-   full markdown content

Initial implementation may rely on simple text scanning.

------------------------------------------------------------------------

# Phase 7 --- Workflow Improvements

After the core system is stable, improvements can be added.

## Possible enhancements

-   interactive inbox processing
-   fuzzy task selection
-   dashboard‑style task views
-   project filtering
-   task statistics
-   LLM‑based task analysis
-   MCP server integration

These features should remain **optional extensions** rather than core
complexity.

------------------------------------------------------------------------

# Phase 8 --- Remove Legacy Behavior

Once the new workflow is stable:

1.  remove the old `open/closed` lifecycle
2.  remove obsolete CLI commands
3.  simplify internal abstractions

The final system should revolve entirely around the **queue model**.

------------------------------------------------------------------------

# Milestone Summary

  Milestone   Outcome
  ----------- ----------------------------------
  Phase 1     Documentation and queue concepts
  Phase 2     New task schema
  Phase 3     Core CLI operations
  Phase 4     Completion workflow
  Phase 5     Obsidian integration
  Phase 6     Editing and search
  Phase 7     Workflow improvements
  Phase 8     Legacy removal

------------------------------------------------------------------------

# Expected Result

At the end of this roadmap, `tqs` will be a **markdown‑native
queue‑based task manager** that:

-   integrates naturally with Obsidian
-   supports terminal‑first workflows
-   stores tasks as simple markdown files
-   enables automation and LLM integrations

The codebase will remain small, understandable, and easy to extend.
