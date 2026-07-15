step 1: sync clone against global canon holding global-only rule/agent/stage/pipeline
synced 13 files (0 written, 13 unchanged, 0 removed)
step 2: global bytes in committed surfaces?
0
none - excluded
step 3: check reports every exclusion:
4
step 4: fresh machine sync --check:
sync check passed (13 generated files)
exit: 0
step 5: project pipeline referencing global-only stage fails validation:
error: project pipeline `shipwreck` references global-only stage `deploy`; copy the stage into `.base/canon/pipelines/stages/` to adopt it
exit: 1
