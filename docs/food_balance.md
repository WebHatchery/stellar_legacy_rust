# Departure food reserve validation

User testing reported 12,000 food against a recommended reserve of 500. New campaigns now start with 1,500 food, three times that baseline reserve. Existing saves retain their stores. Forecasting still accounts for current production, consumption, route tolls and duration; it does not promise protection from future events, deterioration or population change.

The deterministic `smaller_departure_reserve_keeps_starter_and_long_voyages_viable` test compares the old and new stocks on the same 288 combinations: six starter/long charter selections, three legacies, and seeds 0–15. It uses the existing maintenance-oriented autoplay policy, including legal affordable decisions and maintenance. These are automated policy results, not human playtest success rates.

| Starting food | Completed with score at least 0.45 | Dynasty extinctions | Mean ending food |
| --- | --- | --- | --- |
| 12,000 | 288 / 288 | 0 | 9,744 |
| 1,500 | 288 / 288 | 0 | 6,865 |

The lower reserve removes excessive starting inventory without reducing viable outcomes in this cohort. Production can still accumulate a surplus during long voyages. Further human testing should assess how often players notice and act on food risk, rather than treating a large ending stock alone as proof that consumption must increase.


## Ship-work comparison (0.2.1, 2026-09-07)

The earlier reserve table above is historical evidence from the reserve change,
not a fresh claim about the revised ship-work policies. The new 864-run comparison
uses its same seeds, legacies, and starter/long charters at the current 1,500-food
start. No-project play completes 252/288; reactive and prepared play each complete
288/288. Lowest food is respectively 0, 106, and 0; every policy reaches zero spare
parts in at least one campaign. The capped 6% hydroponics benefit now affects real
production, while excess food still spoils through the shared calculation.

See [the full measurements and limitations](ship_work_validation.md) and
[the per-run CSV](ship_work_cohort.csv). These results demonstrate value over
neglect, not a survival advantage for preparedness over reactive repairs. Human
attention, choice quality, and long-voyage scarcity perception remain unverified.
