# Own historical-validator RED/GREEN ledger

Historical runtime proof remains at d2dfdb4a4. Validator GREEN is targeted development proof at a0a29ee21, not a current-head or broad acceptance gate. No original files are tampered.

## validator-red

- argv: `["python3", "-B", "tools/test_patch_4_0_1_validator.py"]`
- cwd: `/home/osso/.worktrees/wow-ui-sim-p401-source`
- revision: `567654d75`
- scope: uncommitted validator tests with generated snapshot; validate.py not implemented
- exit: 1; log: `validator-red.log`
- log SHA-256: `e7c8bc057143d83a2b566893cc7e7dbd6ae3c9f32302fdf2554a64d626d5d641`
- test SHA-256: `99d079cc8e3c34fbd4db78d1f01924a2eebd41c95e34b3d3f188274b1da6f834`

## validator-green

- argv: `["python3", "-B", "tools/test_patch_4_0_1_validator.py"]`
- cwd: `/home/osso/.worktrees/wow-ui-sim-p401-source`
- revision: `a0a29ee21c354194e8fafa1bb107c3514b37ff0b`
- scope: own validator behavioral GREEN/tamper fixtures only; Git absent from child PATH
- exit: 0; log: `validator-green.log`
- log SHA-256: `5d86d188a0e0f6f9bc9535669dd149da1db6be7a61dc993b50562b15e7a64789`
- test SHA-256: `bf547d730b112c1d751cb3431416ae06c66ff7711db1df3577d28a57ed27a5c7`

- historical context SHA-256: `a7ae9cdfb4506ff99f15e4254ddc464fe11723b9563aa662f7d26b446c7c92c0`

GREEN: five fixtures pass. Clean historical validation and unrelated later-state validation reproduce the own source register/extract; source-response/log/archive tamper fixtures reject at exact seals with Git unavailable in subprocess PATH. Original evidence remains byte-identical.
