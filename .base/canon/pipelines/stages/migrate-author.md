---
id: migrate-author
description: Scaffold the pack and author its canon as a rewrite, not a copy.
---

Run `base pack new <id>` to scaffold the library pack, then author each canonical definition the
approved mapping calls for. Rewrite intent into vendor-neutral canon; never paste source bytes
(D-019/D-023). Carry plugin manifest metadata into `pack.md` provenance. Copy skill resource files
byte-for-byte, but re-author prose. Run `base pack check <path>` as the tight loop until the pack's
manifest, paths, and every frontmatter document validate.
