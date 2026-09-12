# Test coverage rationale

The suite is intentionally broader than five cases for several major features:

- The simulation tick, event resolver, contract scoring, and succession suites
  cover independent deterministic rules across years, phases, gates, and
  migration shapes. Combining them would hide which invariant failed.
- The content suites inspect separate registries and authored dimensions
  (charters, peoples, events, voice, ship systems, and campaign beats). Their
  failures point directly to the data contract that needs repair.
- The UI suites cover separate responsive layouts and high-risk controls. They
  stay pure and table-driven where inputs share a rule, while distinct desktop,
  compact, modal, and accessibility boundaries remain separate.

All legacy tests now live under `tests/unit/` and are included by the library
module that owns the private seam they exercise. New tests should prefer the
public library surface and keep each new feature near the five-case target.
