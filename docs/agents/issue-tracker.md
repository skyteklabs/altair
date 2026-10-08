# Issue tracker: Linear

Issues and specs for this repo live in Linear, team `SKY`, project `altair-40f79d2d9973`. Use the Linear MCP server's tools for every operation; don't scrape the web UI.

## Conventions

- **Create an issue**: create it in team `SKY`, project `altair-40f79d2d9973`, with a markdown description.
- **Read an issue**: fetch it by identifier (e.g. `SKY-123`), including its comments and labels.
- **List issues**: list issues filtered by team `SKY` and project `altair-40f79d2d9973`, plus the relevant state and label filters.
- **Comment on an issue**: add a comment to the issue.
- **Apply / remove labels**: add or remove team labels (see `triage-labels.md`). If a label is missing, create it once; don't invent synonyms.
- **Close**: move the issue to a Done workflow state (Canceled for `wontfix`) with a closing comment.

## When a skill says "publish to the issue tracker"

Create a Linear issue in team `SKY`, project `altair-40f79d2d9973`.

## When a skill says "fetch the relevant ticket"

Fetch the Linear issue by identifier, including its comments.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a single parent issue with **child** issues as tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Notes / Decisions-so-far / Fog body.
- **Child ticket**: a sub-issue of the map. Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: Linear "blocked by" issue relations. A ticket is unblocked when every blocker is in a completed or canceled state.
- **Frontier query**: list the map's open sub-issues, drop any with an open blocker or an assignee; first in sub-issue order wins.
- **Claim**: assign the issue to the current user; this is the session's first write.
- **Resolve**: comment the answer, move the issue to Done, then append a context pointer (gist + link) to the map's Decisions-so-far.
