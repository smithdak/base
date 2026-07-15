# Sources

1. **`docs/DECISIONS.md` (D-002, D-007, D-017, D-018, D-019)** — repo intent doc. Constrains
   what a pack may be: domain-neutral core untouched, global layer is seed-and-adopt, committed
   bytes always repo-resident, skillsmith boundary. Weight: **high** (binding constraints, not
   opinions).

2. **`.base/work/W-0007-project-type-packs/item.md`** — the work item this run feeds. Names the
   open questions (pack source location; manual copy vs. CLI helper) and the acceptance bar.
   Weight: **high**.

3. **`C:/Users/dakot/.claude/skills/sitecore-architect/SKILL.md`** — Dakota's authored
   architect-level skill: XM Cloud vs. XP topology decision table, migration phasing, multi-site
   SXA guidance, governance/security/operability checklist, performance review order. Weight:
   **high** for selecting pack content — it is the user's own current best knowledge, already
   distilled. Observed fact: it exists and is user-scoped; inference: its *always-on* subset is
   what a Sitecore repo wants as rules.

4. **`C:/Users/dakot/.claude/plugins/cache/twofoldtech-plugins/plugin-cms-toolkit/1.0.1/skills/sitecore/SKILL.md`**
   — installed third-party developer-level skill (v1.0.1): template inheritance hierarchy
   (base → feature → page), Standard Values discipline, field naming/type conventions, Helix
   layering, serialization. Weight: **medium-high** (adopted by installation, not authored by
   Dakota; content is orthodox Sitecore practice).

5. **Model background knowledge** of the Sitecore ecosystem (SCS serialization, Experience Edge,
   Content SDK direction). Weight: **low** — used only to corroborate, never as sole support;
   version-sensitive claims are excluded from findings per source 3's own drift warning.

## Conflicts

None material. Sources 3 and 4 partition cleanly (architecture vs. mechanics) and agree where
they overlap (SXA for multi-site, Standard Values discipline, serialization as source of truth).
