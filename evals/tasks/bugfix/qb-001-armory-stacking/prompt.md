Players who have both Giant's Might (a god item granting +150% early damage)
and a maxed Haven Armory (+100% damage) report that their damage dropped
after a recent refactor. With both bonuses active they now hit for about
3.5x base damage; before the refactor — and per the documented combat
pipeline — the two bonuses are supposed to compound to 5x.

The documented player damage pipeline is:

    base damage -> Giant's Might % (early_damage_percent)
                -> Haven Armory % (damage_percent)
                -> prestige flat damage -> ascension multiplier
                -> enemy defense -> min 1 -> crit (2x)

Each bonus on its own appears to work (150% alone gives 2.5x, 100% alone
gives 2x). Only the combination is wrong.

Fix the damage calculation so the two percentage bonuses compound the way
the pipeline documents.
