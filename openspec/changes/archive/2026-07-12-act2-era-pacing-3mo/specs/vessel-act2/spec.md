# Delta: vessel-act2 — Era Pacing: 3-Month Balanced Campaign

## MODIFIED Requirements

### Requirement: Ferry-Era Balance Envelope

The ferry era's pacing SHALL stay inside coarse, CI-asserted bands (deterministic simulation, headroom deliberately wide so only structural regressions trip them). Under a balanced yard spend with full-charge jumps, an era SHALL complete in 15–30 crossings spanning 2.5–4.5 real months and deliver at least 84% of the world's 100,000 souls. The naive extremes SHALL remain traps: a Drive-only spend SHALL save no more than 74%. A Ward-leaning spend SHALL save at least 90%, trading a longer era for it. Jumping at full Riftglass charge SHALL never save fewer souls than always jumping at 0% charge.

#### Scenario: The balanced line holds the campaign shape

- **WHEN** a full era is simulated with the balanced spend policy and full-charge jumps
- **THEN** it completes in 15–30 crossings within 2.5–4.5 real months with ≥84% of souls delivered

#### Scenario: Skill is rewarded, not marginal

- **WHEN** full eras are simulated with a Ward-leaning spend and with a Drive-only spend
- **THEN** the Ward-leaning line delivers ≥90% of souls and the Drive-only line delivers ≤74%

#### Scenario: Patience at the Dock pays

- **WHEN** full eras are simulated jumping always at 100% charge and always at 0% charge
- **THEN** the full-charge era delivers at least as many souls as the 0%-charge era
