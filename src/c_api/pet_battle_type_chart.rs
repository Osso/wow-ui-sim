//! INFERRED public pet-type chart; R88f.D.61 establishes only the native matrix.
//! Sources: https://warcraft.wiki.gg/wiki/Pet_Battle_System#Types and
//! https://warcraft.wiki.gg/wiki/API_C_PetBattles.GetAttackModifier (weak .666667).
const WEAK: f64 = 2.0 / 3.0;
// Rows attack, columns defend: Humanoid, Dragonkin, Flying, Undead, Critter,
// Magic, Elemental, Beast, Aquatic, Mechanical.
const TYPE_MODIFIERS: [[f64; 10]; 10] = [
    [1.0, 1.5, 1.0, 1.0, 1.0, 1.0, 1.0, WEAK, 1.0, 1.0],
    [1.0, 1.0, 1.0, WEAK, 1.0, 1.5, 1.0, 1.0, 1.0, 1.0],
    [1.0, WEAK, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.5, 1.0],
    [1.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, WEAK, 1.0],
    [WEAK, 1.0, 1.0, 1.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, WEAK],
    [1.0, 1.0, 1.0, 1.0, WEAK, 1.0, 1.0, 1.0, 1.0, 1.5],
    [1.0, 1.0, WEAK, 1.0, 1.5, 1.0, 1.0, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0, 1.0, WEAK, 1.5, 1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, WEAK, 1.5, 1.0, 1.0],
];

pub(super) fn attack_modifier(attacker: i32, defender: i32) -> f64 {
    let index = |value: i32| value.checked_sub(1).and_then(|v| usize::try_from(v).ok());
    index(attacker)
        .and_then(|row| TYPE_MODIFIERS.get(row))
        .and_then(|row| index(defender).and_then(|column| row.get(column)))
        .copied()
        // INFERRED retained invalid-type compatibility; native errors unresolved.
        .unwrap_or(1.0)
}
