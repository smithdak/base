# Outline: Sitecore pack v1

**Audience:** a coding agent (any harness) working in a Sitecore repository that has adopted the
pack, and Dakota deciding what to adopt. **Job:** give the agent the always-on constraints and
on-demand reference a Sitecore project requires, without duplicating user-level skills.
**Destination:** `~/.base/canon/packs/sitecore/`. **Voice:** same register as existing canon —
imperative, compact, no marketing. **Length:** each file ≤ ~1 page; the pack is a seed, not a book.

## Files

1. `pack.md` — manifest. Sections: what the pack is (2–3 sentences), inventory table (file →
   kind → what it covers), adoption instructions (copy which folders where, add INDEX lines,
   run `base sync`), provenance note (distilled from which sources, date).
2. `rules/sitecore-conventions.md` — always-on constraints, one bulleted list ordered by blast
   radius: serialization as source of truth; Standard Values mandatory; Helix boundaries;
   secrets never serialized; field naming. Each bullet one sentence, imperative.
3. `knowledge/sitecore-content-modeling.md` — template hierarchy diagram (base → feature →
   page), Standard Values checklist, field naming/type table. Mechanics an agent needs when
   modeling content, not before.
4. `knowledge/sitecore-platform-decisions.md` — XM Cloud vs. XP decision signals (condensed
   table), topology cheatsheet for self-hosted, migration phases with the two standard
   under-scoping red flags (head rebuild, personalization re-architecture).
5. `knowledge/sitecore-governance-operability.md` — governance (workflow, insert options,
   shared-vs-local boundary), security hardening list, CI/CD expectations (serialization in the
   pipeline), observability/DR ownership split for XM Cloud.

## Gate: plan-approval

Standing approval: the user set a session goal directive (`/goal w-0006 and w-0007`) explicitly
instructing autonomous completion of these work items without pausing; that directive is this
outline's approval. Recorded here per the gate's requirement for explicit, traceable approval.
