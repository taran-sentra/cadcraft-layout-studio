# CADCraft: Layout studio build

This is [storytold/cadcraft](https://github.com/storytold/cadcraft) plus a few changes a
plotted-layout drafting team needed. The branch `layout-studio` is upstream `main` with these
commits on top. It is rebased on upstream every day, so it always has the latest CADCraft.

| Change | What it does |
|---|---|
| Hatch area | Properties > Geometry shows a hatch's Area (islands deducted, as AREA gives it). |
| Cumulative area | Several closed polylines or hatches selected: Properties shows their Cumulative area. |
| TRIM / BREAK on XLINE and RAY | As AutoCAD does: an xline becomes rays, a ray a line and a ray (upstream issue #1235). |

**Download (Windows, no install):**
[CADCraft-patched-windows-x64.zip](../../releases/latest/download/CADCraft-patched-windows-x64.zip),
always the newest build. Extract it and double-click `CADCraft.exe`. The build is not
code-signed: on "Windows protected your PC" choose More info > Run anyway.

When a rebase hits a conflict, the branch stays at its last good state and an issue is opened
on this repository.

Licence: the same as CADCraft (MIT or Apache-2.0, see `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`).
