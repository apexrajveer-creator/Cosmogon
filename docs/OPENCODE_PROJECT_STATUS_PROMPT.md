# Prompt for OpenCode: Produce a complete Cosmogon status report

You are auditing the current Cosmogon repository so another assistant can accurately continue the work and update a public project article. Produce **one complete Markdown report** at:

`docs/OPENCODE_PROJECT_STATUS.md`

Do not modify source code, the article draft, README, release files, or any other project content. Do not commit, push, tag, publish, or release anything. Read-only Git/GitHub inspection is permitted. If you need credentials or access you do not have, state the limitation instead of asking the owner to expose secrets.

## Start by establishing the actual state

Read `AGENTS.md`, `HANDOFF.md`, `docs/CONTINUATION_HANDOFF.md`, `docs/MASTER_DEVELOPMENT_DIRECTIVE.md`, `docs/MASTER_GAP_ANALYSIS.md`, the current `CHANGELOG.md`, and relevant validation/design documents. Inspect the current branch, `HEAD`, worktree status, remotes, upstream tracking, local branches, tags, and commit graph. Treat repository evidence as authoritative over old handoff summaries when they conflict, and call out the discrepancy.

Determine whether **all current project work is present on GitHub**. Compare the local branch and all relevant local commits with the live remote using read-only fetch/list operations or the GitHub website/API. Check the default branch, active development branch, latest public release/tag, and whether current local commits or working-tree changes are missing from GitHub. Do not infer remote state from stale `origin/*` refs alone. If the network or access prevents live verification, clearly say that the GitHub synchronization question remains unverified and give the exact local evidence you do have. Never push as part of this audit.

## Report what exists now, not just what shipped in 1.0

Create a concise but thorough feature inventory organized into:

1. **Released 1.0 capabilities** — identify the exact release/tag and its scope.
2. **Implemented after 1.0** — group the substantial additions by product area, with commit IDs, key files, and validation documents. Include research controls/recordings/replay, numerical and scientific validation, Mars source-backed terrain and studies, habitat/greenhouse/crop/thermal/moisture systems and controlled experiments, navigation/camera/audio/input work, and spacecraft/vehicle/ground-contact/supply research as applicable to the actual checkout.
3. **Current uncommitted work** — enumerate each relevant modified/untracked file and explain what it adds, its verification state, and whether it is integrated into authoritative simulation/save/replay state.
4. **Not implemented or still partial** — distinguish working foundations from end-to-end product capabilities. Include the next dependency sequence from the active handoff and the user's detailed animated close-up graphics requirement for every visitable cosmic body.

Avoid listing every file or every tiny commit when a grouped feature summary is clearer. Do not claim a planned, designed, or partially implemented system is complete.

## Verify engineering and scientific evidence separately

For each test or benchmark figure, identify the exact commit/source snapshot and supporting validation document. State whether evidence applies to committed code or the current dirty tree. Report test/clippy/build/capture results only when repository artifacts or a fresh read-only verification prove them. Do not run expensive builds or mutate files without need.

Separate these concepts explicitly:

- deterministic/reproducible execution;
- software regression tests;
- numerical checks against analytic cases;
- comparisons against external observations or reference data;
- independent scientific validation and calibration.

Explain what the existing evidence does and does not validate. Include important uncertainty, source, licensing, frame/registration, model-assumption, and accuracy gaps. Never describe all of Cosmogon as research-grade or empirically validated merely because it has research panels, exact exports, deterministic replay, or passing tests.

## Assess the article draft

Review `docs/articles/cosmogon-project-overview.md` and report whether it represents the current repository accurately. Identify stale release/test figures, missing major post-1.0 work, incorrect or unsupported claims, source/reference gaps, and any licensing/byline issues. Recommend concrete changes, but **do not edit the article**. Clearly distinguish a public project overview from a focused scholarly paper and say whether the current evidence supports either one yet.

## Estimate remaining work responsibly

Use the actual master directive and gap analysis to summarize remaining work by major phase or product area. Include the immediate next build and its dependencies. If reporting a percentage or time estimate, label it as a rough judgment, state its basis and uncertainty, and do not present it as measured progress. Say plainly if the full directive remains incomplete.

## Format and evidence rules

Save the report as valid Markdown with:

- an executive status at the top, including audit date, branch, `HEAD`, worktree cleanliness, release status, and GitHub synchronization result;
- clear headings for release baseline, post-1.0 work, current dirty work, validation, scientific limits, article review, remaining work, and handoff priorities;
- repository-relative file links and commit IDs for claims, with line numbers where practical;
- direct links to primary/official external sources only where those support a factual claim;
- a final section titled **What the next assistant should do first** with a short ordered checklist.

Use exact dates and version identifiers. Do not invent the owner's public name, affiliation, user adoption, research impact, publication status, patentability, or validation. Mark unknowns as unknown. At the end, report the path written and summarize any question you could not verify.
