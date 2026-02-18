## 2024-05-22 - SolidJS Label Association
**Learning:** Found a pattern where custom Input components were rendering labels but not associating them via `for`/`id` attributes, breaking screen reader support.
**Action:** When auditing SolidJS components, always check that `splitProps` handles `id` correctly and that `label` uses `for` (not `htmlFor`) to point to the input's ID.
