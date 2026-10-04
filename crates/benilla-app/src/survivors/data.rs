//! The mode's tables: classes, abilities, passives, enemies and the wave clock. Spell ids name
//! `Spell.dbc` rows only for their icon, name and visual chain; every number here is the mode's.

/// A weapon on the hero's virtual item slots: `(slot, display, class, subclass, material, inv,
/// sheath)`, the creature form of `UNIT_VIRTUAL_ITEM_SLOT_DISPLAY` and `_INFO`.
pub(crate) type Weapon = (u16, u32, u8, u8, u8, u8, u8);

pub(crate) struct ClassDef {
    pub id: u8,
    pub name: &'static str,
    /// `RAID_CLASS_COLORS`, 0xRRGGBB.
    pub color: u32,
    /// `UNIT_FIELD_BYTES_0` byte 3.
    pub power: u8,
    pub start: &'static str,
    pub pool: &'static [&'static str],
    /// Class-trainer `CreatureDisplayInfo` ids, deduped by race and sex at the menu.
    pub looks: &'static [u32],
    pub weapons: &'static [Weapon],
    /// The sheath state the hero fights in: 1 melee, 2 ranged.
    pub sheath: u8,
    pub blurb: &'static str,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Kind {
    /// A missile at the nearest enemies (or instant if the spell has no speed).
    Bolt,
    /// An instant hit on the nearest enemies in range.
    Strike,
    /// A close hit on the nearest enemies in melee reach.
    Melee,
    /// Everything in a radius around the hero.
    Nova,
    /// Everything in a cone toward the nearest enemy.
    Cone,
    /// A ground area at the densest nearby enemy, ticking for `duration`.
    Ground,
    /// A ground area under the hero, ticking for `duration`.
    GroundSelf,
    /// Jumps from enemy to enemy.
    Chain,
    /// A damage-over-time on several enemies.
    Dot,
    /// Always on: pulses every `cooldown` around the hero, with the spell's state visual on him.
    Aura,
    /// Always on: zaps one enemy close to the hero every `cooldown`.
    Orbit,
    /// Absorbs `damage` for `duration`.
    Shield,
    /// Immune for `duration`.
    Immune,
    /// Returns `damage` to every attacker for `duration`.
    Thorns,
    /// Heals `damage` over `duration`.
    Heal,
    /// Drops a totem that casts `sub` for `duration`.
    Totem,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Effect {
    None,
    /// Speed multiplier for seconds.
    Slow(f32, f32),
    Root(f32),
    Stun(f32),
    Fear(f32),
    /// Outgoing damage multiplier for seconds.
    Weaken(f32, f32),
    /// Heal the hero by this fraction of damage dealt.
    Drain(f32),
    /// Kill outright below this health fraction.
    Execute(f32),
}

pub(crate) struct AbilityDef {
    pub key: &'static str,
    pub spell: u32,
    pub kind: Kind,
    /// Direct damage; per tick for areas and pulses; absorb, heal or reflect for the self kinds.
    pub damage: f32,
    /// Damage over time, per second, for `duration`.
    pub dot: f32,
    pub cooldown: f32,
    pub range: f32,
    /// Splash for single hits, area for the area kinds, the jump for a chain.
    pub radius: f32,
    pub count: u32,
    pub duration: f32,
    pub effect: Effect,
    /// The spell whose state kit marks an affected enemy, 0 for none.
    pub mark: u32,
    /// A totem's display, and the spell it casts.
    pub totem: (u32, u32),
    /// The area's own visual hangs overhead and would hide the field from the camera, so the
    /// area plays the spell's impact on whatever it hits instead.
    pub overhead: bool,
    pub desc: &'static str,
}

const fn ab(key: &'static str, spell: u32, kind: Kind) -> AbilityDef {
    AbilityDef {
        key,
        spell,
        kind,
        damage: 0.0,
        dot: 0.0,
        cooldown: 2.0,
        range: 20.0,
        radius: 0.0,
        count: 1,
        duration: 0.0,
        effect: Effect::None,
        mark: 0,
        totem: (0, 0),
        overhead: false,
        desc: "",
    }
}

macro_rules! ability {
    ($key:literal, $spell:literal, $kind:ident { $($f:ident: $v:expr),* $(,)? }) => {
        AbilityDef { $($f: $v,)* ..ab($key, $spell, Kind::$kind) }
    };
}

pub(crate) static ABILITIES: &[AbilityDef] = &[
    // --- Mage ---
    ability!(
        "frostbolt",
        116,
        Bolt {
            damage: 12.0,
            cooldown: 1.3,
            range: 24.0,
            effect: Effect::Slow(0.5, 2.5),
            mark: 116,
            desc: "Hurls a bolt of frost at the nearest enemy, slowing it."
        }
    ),
    ability!(
        "fireball",
        133,
        Bolt {
            damage: 20.0,
            cooldown: 2.4,
            range: 26.0,
            radius: 3.5,
            desc: "A ball of fire that bursts on impact, burning nearby enemies."
        }
    ),
    ability!(
        "arcane_missiles",
        7268,
        Bolt {
            damage: 6.0,
            cooldown: 1.9,
            range: 22.0,
            count: 3,
            desc: "Launches arcane missiles at several nearby enemies."
        }
    ),
    ability!(
        "frost_nova",
        122,
        Nova {
            damage: 8.0,
            cooldown: 7.0,
            radius: 9.0,
            effect: Effect::Root(2.5),
            mark: 122,
            desc: "Blasts enemies around you with frost, freezing them in place."
        }
    ),
    ability!(
        "arcane_explosion",
        1449,
        Nova {
            damage: 10.0,
            cooldown: 2.8,
            radius: 7.0,
            desc: "Arcane energy erupts around you."
        }
    ),
    ability!(
        "blizzard",
        10,
        Ground {
            damage: 7.0,
            cooldown: 9.0,
            range: 20.0,
            radius: 7.0,
            duration: 6.0,
            effect: Effect::Slow(0.6, 1.5),
            desc: "Ice shards rain on the thickest pack, slowing everything inside."
        }
    ),
    ability!(
        "flamestrike",
        2120,
        Ground {
            damage: 14.0,
            cooldown: 8.0,
            range: 20.0,
            radius: 5.0,
            duration: 4.0,
            desc: "Calls down a pillar of fire that keeps burning."
        }
    ),
    ability!(
        "fire_blast",
        2136,
        Strike {
            damage: 26.0,
            cooldown: 3.5,
            range: 18.0,
            desc: "Blasts the nearest enemy with fire."
        }
    ),
    ability!(
        "cone_of_cold",
        120,
        Cone {
            damage: 14.0,
            cooldown: 4.5,
            radius: 11.0,
            effect: Effect::Slow(0.5, 3.0),
            mark: 120,
            desc: "A cone of frost toward the closest enemy."
        }
    ),
    ability!(
        "blast_wave",
        11113,
        Nova {
            damage: 18.0,
            cooldown: 6.5,
            radius: 8.0,
            effect: Effect::Slow(0.6, 2.0),
            desc: "A wave of flame bursts out from you."
        }
    ),
    ability!(
        "pyroblast",
        11366,
        Bolt {
            damage: 70.0,
            dot: 6.0,
            duration: 6.0,
            cooldown: 6.0,
            range: 30.0,
            desc: "A huge fiery boulder at the toughest enemy in range."
        }
    ),
    // --- Warrior ---
    ability!(
        "cleave",
        845,
        Melee {
            damage: 14.0,
            cooldown: 1.1,
            range: 5.5,
            count: 3,
            desc: "Strikes the closest enemies in front of you."
        }
    ),
    ability!(
        "heroic_strike",
        78,
        Melee {
            damage: 28.0,
            cooldown: 1.6,
            range: 5.5,
            desc: "A mighty blow on the closest enemy."
        }
    ),
    ability!(
        "whirlwind",
        1680,
        Nova {
            damage: 16.0,
            cooldown: 4.0,
            radius: 7.0,
            desc: "Spins, striking every enemy around you."
        }
    ),
    ability!(
        "thunder_clap",
        6343,
        Nova {
            damage: 10.0,
            cooldown: 5.0,
            radius: 8.0,
            effect: Effect::Slow(0.5, 3.0),
            mark: 6343,
            desc: "Thunder slows and damages enemies around you."
        }
    ),
    ability!(
        "sweeping_strikes",
        12292,
        Aura {
            damage: 6.0,
            cooldown: 0.6,
            radius: 5.5,
            desc: "Your blows carry into every enemy close around you."
        }
    ),
    ability!(
        "demoralizing_shout",
        1160,
        Nova {
            damage: 4.0,
            cooldown: 8.0,
            radius: 11.0,
            effect: Effect::Weaken(0.5, 5.0),
            desc: "Halves the damage of enemies around you."
        }
    ),
    ability!(
        "execute",
        5308,
        Strike {
            damage: 40.0,
            cooldown: 2.0,
            range: 8.0,
            count: 2,
            effect: Effect::Execute(0.3),
            desc: "Finishes off weakened enemies outright."
        }
    ),
    ability!(
        "mortal_strike",
        12294,
        Melee {
            damage: 48.0,
            cooldown: 3.0,
            range: 6.0,
            desc: "A vicious strike on the closest enemy."
        }
    ),
    ability!(
        "retaliation",
        20230,
        Thorns {
            damage: 14.0,
            cooldown: 18.0,
            duration: 8.0,
            desc: "Strikes back at every enemy that hits you."
        }
    ),
    // --- Rogue ---
    ability!(
        "sinister_strike",
        1752,
        Melee {
            damage: 15.0,
            cooldown: 0.85,
            range: 5.5,
            desc: "A quick strike on the closest enemy."
        }
    ),
    ability!(
        "eviscerate",
        2098,
        Melee {
            damage: 42.0,
            cooldown: 2.8,
            range: 5.5,
            desc: "A finishing move that tears into the closest enemy."
        }
    ),
    ability!(
        "blade_flurry",
        13877,
        Nova {
            damage: 11.0,
            cooldown: 3.0,
            radius: 6.0,
            desc: "Your blades lash out at every nearby enemy."
        }
    ),
    ability!(
        "rupture",
        1943,
        Dot {
            dot: 5.0,
            cooldown: 3.5,
            range: 10.0,
            count: 3,
            duration: 6.0,
            desc: "Opens bleeding wounds on several enemies."
        }
    ),
    ability!(
        "gouge",
        1776,
        Melee {
            damage: 8.0,
            cooldown: 4.0,
            range: 6.0,
            count: 2,
            effect: Effect::Stun(2.0),
            mark: 1776,
            desc: "Gouges the closest enemies, stunning them."
        }
    ),
    ability!(
        "kidney_shot",
        408,
        Melee {
            damage: 12.0,
            cooldown: 6.0,
            range: 6.0,
            effect: Effect::Stun(3.5),
            mark: 408,
            desc: "Stuns the closest enemy."
        }
    ),
    // --- Priest ---
    ability!(
        "smite",
        585,
        Strike {
            damage: 14.0,
            cooldown: 1.2,
            range: 20.0,
            desc: "Smites the nearest enemy with holy light."
        }
    ),
    ability!(
        "shadow_word_pain",
        589,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 18.0,
            count: 3,
            duration: 9.0,
            desc: "Shadow pain on several enemies over time."
        }
    ),
    ability!(
        "holy_fire",
        14914,
        Strike {
            damage: 22.0,
            dot: 3.0,
            duration: 6.0,
            cooldown: 3.5,
            range: 20.0,
            desc: "Holy flame burns an enemy, and keeps burning."
        }
    ),
    ability!(
        "mind_blast",
        8092,
        Strike {
            damage: 32.0,
            cooldown: 3.5,
            range: 22.0,
            desc: "Blasts an enemy's mind for heavy shadow damage."
        }
    ),
    ability!(
        "holy_nova",
        15237,
        Nova {
            damage: 11.0,
            cooldown: 3.2,
            radius: 9.0,
            effect: Effect::Drain(0.1),
            desc: "Holy energy bursts from you, healing you as it burns."
        }
    ),
    ability!(
        "psychic_scream",
        8122,
        Nova {
            damage: 2.0,
            cooldown: 12.0,
            radius: 8.0,
            effect: Effect::Fear(3.5),
            mark: 8122,
            desc: "Terrifies nearby enemies into fleeing."
        }
    ),
    ability!(
        "power_word_shield",
        17,
        Shield {
            damage: 30.0,
            cooldown: 14.0,
            duration: 10.0,
            desc: "A shield that absorbs damage."
        }
    ),
    // --- Warlock ---
    ability!(
        "shadow_bolt",
        686,
        Bolt {
            damage: 15.0,
            cooldown: 1.4,
            range: 24.0,
            desc: "Sends a shadowy bolt at the nearest enemy."
        }
    ),
    ability!(
        "corruption",
        172,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 20.0,
            count: 3,
            duration: 10.0,
            mark: 172,
            desc: "Corrupts several enemies with shadow over time."
        }
    ),
    ability!(
        "curse_of_agony",
        980,
        Dot {
            dot: 3.5,
            cooldown: 2.5,
            range: 20.0,
            count: 4,
            duration: 12.0,
            desc: "Curses several enemies with lasting agony."
        }
    ),
    ability!(
        "immolate",
        348,
        Strike {
            damage: 12.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 2.4,
            range: 20.0,
            count: 2,
            mark: 348,
            desc: "Sets enemies ablaze."
        }
    ),
    ability!(
        "rain_of_fire",
        5740,
        Ground {
            damage: 8.0,
            cooldown: 9.0,
            range: 20.0,
            radius: 7.0,
            duration: 6.0,
            desc: "Fire rains on the thickest pack of enemies."
        }
    ),
    ability!(
        "hellfire",
        5857,
        Aura {
            damage: 6.0,
            cooldown: 1.0,
            radius: 7.0,
            desc: "Constantly burns everything around you."
        }
    ),
    ability!(
        "death_coil",
        6789,
        Bolt {
            damage: 26.0,
            cooldown: 5.0,
            range: 22.0,
            effect: Effect::Drain(0.5),
            desc: "A shadow coil that heals you for half its damage."
        }
    ),
    ability!(
        "searing_pain",
        5676,
        Strike {
            damage: 18.0,
            cooldown: 1.8,
            range: 20.0,
            desc: "Inflicts searing pain on the nearest enemy."
        }
    ),
    ability!(
        "shadowburn",
        17877,
        Strike {
            damage: 46.0,
            cooldown: 5.0,
            range: 20.0,
            desc: "Instantly blasts an enemy with shadow."
        }
    ),
    // --- Hunter ---
    ability!(
        "arcane_shot",
        3044,
        Bolt {
            damage: 13.0,
            cooldown: 1.0,
            range: 28.0,
            desc: "An arcane arrow at the nearest enemy."
        }
    ),
    ability!(
        "multi_shot",
        2643,
        Bolt {
            damage: 10.0,
            cooldown: 2.0,
            range: 26.0,
            count: 3,
            desc: "Fires arrows at several enemies."
        }
    ),
    ability!(
        "serpent_sting",
        1978,
        Bolt {
            damage: 2.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 2.6,
            range: 26.0,
            count: 2,
            desc: "Stings enemies with poison over time."
        }
    ),
    ability!(
        "volley",
        1510,
        Ground {
            damage: 8.0,
            cooldown: 9.0,
            range: 24.0,
            radius: 7.0,
            duration: 5.0,
            desc: "Arrows rain on the thickest pack of enemies."
        }
    ),
    ability!(
        "explosive_trap",
        13812,
        Ground {
            damage: 22.0,
            dot: 4.0,
            cooldown: 8.0,
            range: 14.0,
            radius: 5.0,
            duration: 0.0,
            desc: "A trap explodes under a pack, burning them."
        }
    ),
    ability!(
        "concussive_shot",
        5116,
        Bolt {
            damage: 8.0,
            cooldown: 3.0,
            range: 28.0,
            effect: Effect::Slow(0.5, 4.0),
            mark: 5116,
            desc: "Dazes an enemy, slowing it."
        }
    ),
    ability!(
        "aimed_shot",
        19434,
        Bolt {
            damage: 48.0,
            cooldown: 4.0,
            range: 32.0,
            desc: "A powerful shot at the toughest enemy in range."
        }
    ),
    // --- Shaman ---
    ability!(
        "lightning_bolt",
        403,
        Bolt {
            damage: 15.0,
            cooldown: 1.3,
            range: 24.0,
            desc: "Hurls lightning at the nearest enemy."
        }
    ),
    ability!(
        "chain_lightning",
        421,
        Chain {
            damage: 18.0,
            cooldown: 3.0,
            range: 20.0,
            radius: 9.0,
            count: 3,
            desc: "Lightning that jumps between enemies."
        }
    ),
    ability!(
        "earth_shock",
        8042,
        Strike {
            damage: 22.0,
            cooldown: 2.5,
            range: 20.0,
            desc: "Shocks the nearest enemy with earth."
        }
    ),
    ability!(
        "flame_shock",
        8050,
        Strike {
            damage: 10.0,
            dot: 4.0,
            duration: 9.0,
            cooldown: 3.0,
            range: 20.0,
            count: 2,
            desc: "Instant fire, and a burn after."
        }
    ),
    ability!(
        "frost_shock",
        8056,
        Strike {
            damage: 16.0,
            cooldown: 3.0,
            range: 20.0,
            effect: Effect::Slow(0.5, 4.0),
            desc: "Shocks an enemy with frost, slowing it."
        }
    ),
    ability!(
        "searing_totem",
        3599,
        Totem {
            damage: 9.0,
            cooldown: 14.0,
            range: 20.0,
            duration: 14.0,
            totem: (4589, 3606),
            desc: "A totem that shoots fire at nearby enemies."
        }
    ),
    ability!(
        "magma_totem",
        8190,
        Totem {
            damage: 8.0,
            cooldown: 16.0,
            radius: 7.0,
            duration: 14.0,
            totem: (4683, 8349),
            desc: "A totem that pulses fire around itself."
        }
    ),
    ability!(
        "lightning_shield",
        324,
        Orbit {
            damage: 12.0,
            cooldown: 0.9,
            radius: 4.0,
            desc: "Orbs of lightning zap enemies that come close."
        }
    ),
    // --- Paladin ---
    ability!(
        "hammer_of_wrath",
        24275,
        Bolt {
            damage: 16.0,
            cooldown: 1.5,
            range: 24.0,
            desc: "Hurls a hammer at the nearest enemy."
        }
    ),
    ability!(
        "consecration",
        26573,
        GroundSelf {
            damage: 7.0,
            cooldown: 9.0,
            radius: 7.0,
            duration: 8.0,
            desc: "Consecrates the ground under you."
        }
    ),
    ability!(
        "holy_shock",
        25912,
        Strike {
            damage: 24.0,
            cooldown: 2.5,
            range: 20.0,
            desc: "A burst of holy energy at the nearest enemy."
        }
    ),
    ability!(
        "exorcism",
        879,
        Strike {
            damage: 36.0,
            cooldown: 5.0,
            range: 22.0,
            desc: "Holy power drives out the toughest enemy."
        }
    ),
    ability!(
        "holy_wrath",
        2812,
        Nova {
            damage: 14.0,
            cooldown: 5.0,
            radius: 11.0,
            desc: "Holy bolts strike every enemy around you."
        }
    ),
    ability!(
        "hammer_of_justice",
        853,
        Strike {
            damage: 8.0,
            cooldown: 6.0,
            range: 12.0,
            count: 2,
            effect: Effect::Stun(3.0),
            mark: 853,
            desc: "Stuns enemies with a hammer of justice."
        }
    ),
    ability!(
        "divine_shield",
        642,
        Immune {
            cooldown: 28.0,
            duration: 4.0,
            desc: "Makes you immune to all damage."
        }
    ),
    ability!(
        "judgement",
        20271,
        Strike {
            damage: 26.0,
            cooldown: 4.0,
            range: 14.0,
            count: 2,
            desc: "Judges enemies with holy light."
        }
    ),
    // --- Druid ---
    ability!(
        "wrath",
        5176,
        Bolt {
            damage: 13.0,
            cooldown: 1.2,
            range: 24.0,
            desc: "Hurls nature's wrath at the nearest enemy."
        }
    ),
    ability!(
        "moonfire",
        8921,
        Strike {
            damage: 10.0,
            dot: 3.0,
            duration: 9.0,
            cooldown: 2.0,
            range: 22.0,
            count: 2,
            desc: "Burns enemies with moonlight."
        }
    ),
    ability!(
        "starfire",
        2912,
        Strike {
            damage: 40.0,
            cooldown: 4.0,
            range: 24.0,
            desc: "Star power on the toughest enemy in range."
        }
    ),
    ability!(
        "hurricane",
        16914,
        Ground {
            damage: 7.0,
            cooldown: 10.0,
            range: 20.0,
            radius: 8.0,
            duration: 7.0,
            effect: Effect::Slow(0.6, 1.5),
            overhead: true,
            desc: "A storm batters the thickest pack, slowing it."
        }
    ),
    ability!(
        "insect_swarm",
        5570,
        Dot {
            dot: 4.0,
            cooldown: 3.0,
            range: 20.0,
            count: 3,
            duration: 10.0,
            mark: 5570,
            desc: "Swarms several enemies with insects."
        }
    ),
    ability!(
        "entangling_roots",
        339,
        Dot {
            dot: 2.0,
            cooldown: 6.0,
            range: 16.0,
            count: 4,
            duration: 5.0,
            effect: Effect::Root(4.0),
            mark: 339,
            desc: "Roots several enemies in place."
        }
    ),
    ability!(
        "thorns",
        467,
        Thorns {
            damage: 8.0,
            cooldown: 15.0,
            duration: 15.0,
            desc: "Thorns hurt enemies that strike you."
        }
    ),
    ability!(
        "rejuvenation",
        774,
        Heal {
            damage: 30.0,
            cooldown: 15.0,
            duration: 10.0,
            desc: "Heals you over time."
        }
    ),
];

pub(crate) fn ability(key: &str) -> &'static AbilityDef {
    ABILITIES
        .iter()
        .find(|a| a.key == key)
        .unwrap_or_else(|| panic!("survivors: no ability {key:?}"))
}

pub(crate) const MAX_ABILITY_LEVEL: u8 = 8;
pub(crate) const MAX_PASSIVE_LEVEL: u8 = 5;
pub(crate) const MAX_ABILITIES: usize = 6;
pub(crate) const MAX_PASSIVES: usize = 6;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Stat {
    Might,
    MaxHp,
    Speed,
    Regen,
    Haste,
    Area,
    Magnet,
    Growth,
    Armor,
    Crit,
}

pub(crate) struct PassiveDef {
    pub key: &'static str,
    pub name: &'static str,
    /// Icon source, and a state visual on the hero where the spell has one.
    pub spell: u32,
    pub stat: Stat,
    pub per_level: f32,
    pub desc: &'static str,
}

pub(crate) static PASSIVES: &[PassiveDef] = &[
    PassiveDef {
        key: "might",
        name: "Blessing of Might",
        spell: 19740,
        stat: Stat::Might,
        per_level: 0.10,
        desc: "+10% damage.",
    },
    PassiveDef {
        key: "fortitude",
        name: "Power Word: Fortitude",
        spell: 1243,
        stat: Stat::MaxHp,
        per_level: 0.20,
        desc: "+20% max health.",
    },
    PassiveDef {
        key: "pack",
        name: "Aspect of the Pack",
        spell: 13159,
        stat: Stat::Speed,
        per_level: 0.10,
        desc: "+10% movement speed.",
    },
    PassiveDef {
        key: "renew",
        name: "Renew",
        spell: 139,
        stat: Stat::Regen,
        per_level: 0.5,
        desc: "Regenerate 0.5 health per second.",
    },
    PassiveDef {
        key: "intellect",
        name: "Arcane Intellect",
        spell: 1459,
        stat: Stat::Haste,
        per_level: 0.08,
        desc: "Abilities recharge 8% faster.",
    },
    PassiveDef {
        key: "amplify",
        name: "Amplify Magic",
        spell: 1008,
        stat: Stat::Area,
        per_level: 0.10,
        desc: "+10% area of effect.",
    },
    PassiveDef {
        key: "detect",
        name: "Detect Lesser Invisibility",
        spell: 132,
        stat: Stat::Magnet,
        per_level: 0.40,
        desc: "+40% pickup range.",
    },
    PassiveDef {
        key: "wisdom",
        name: "Blessing of Wisdom",
        spell: 19742,
        stat: Stat::Growth,
        per_level: 0.08,
        desc: "+8% experience gained.",
    },
    PassiveDef {
        key: "wild",
        name: "Mark of the Wild",
        spell: 1126,
        stat: Stat::Armor,
        per_level: 1.0,
        desc: "Reduces damage taken from each hit by 1.",
    },
    PassiveDef {
        key: "kings",
        name: "Blessing of Kings",
        spell: 20217,
        stat: Stat::Crit,
        per_level: 0.05,
        desc: "+5% chance to critically strike for double damage.",
    },
];

pub(crate) static CLASSES: &[ClassDef] = &[
    ClassDef {
        id: 1,
        name: "Warrior",
        color: 0xC79C6E,
        power: 1,
        start: "cleave",
        pool: &[
            "cleave",
            "heroic_strike",
            "whirlwind",
            "thunder_clap",
            "sweeping_strikes",
            "demoralizing_shout",
            "execute",
            "mortal_strike",
            "retaliation",
        ],
        looks: &[
            1300, 1374, 1375, 1504, 1578, 1599, 1707, 1721, 1880, 2096, 2103, 2113, 2196, 2198,
            2614, 2620, 2658, 3053, 3054, 3055, 3280, 3287, 3343, 3399, 3431, 3743, 3793, 3794,
            4242, 4556, 6071, 11037,
        ],
        weapons: &[(0, 7483, 2, 7, 1, 13, 3), (1, 1706, 4, 6, 1, 14, 4)],
        sheath: 1,
        blurb: "Cleaves through the horde up close. Tough, and hits hard.",
    },
    ClassDef {
        id: 2,
        name: "Paladin",
        color: 0xF58CBA,
        power: 0,
        start: "hammer_of_wrath",
        pool: &[
            "hammer_of_wrath",
            "consecration",
            "holy_shock",
            "exorcism",
            "holy_wrath",
            "hammer_of_justice",
            "divine_shield",
            "judgement",
        ],
        looks: &[
            1299, 1499, 1622, 3087, 3088, 3089, 3284, 3289, 3346, 3393, 7356,
        ],
        weapons: &[(0, 8011, 2, 4, 2, 13, 3), (1, 1705, 4, 6, 1, 14, 4)],
        sheath: 1,
        blurb: "Holy hammers and consecrated ground. Hard to kill.",
    },
    ClassDef {
        id: 3,
        name: "Hunter",
        color: 0xABD473,
        power: 0,
        start: "arcane_shot",
        pool: &[
            "arcane_shot",
            "multi_shot",
            "serpent_sting",
            "volley",
            "explosive_trap",
            "concussive_shot",
            "aimed_shot",
        ],
        looks: &[
            1373, 1703, 1723, 1882, 2066, 2087, 2105, 2112, 2205, 2206, 2251, 3056, 3072, 3073,
            3299, 3309, 3310, 3312, 3395, 3558, 3744, 3810, 3811, 4239, 4241, 4372, 4560, 7538,
            10245,
        ],
        weapons: &[(0, 7480, 2, 6, 1, 17, 2), (2, 6235, 2, 2, 2, 15, 0)],
        sheath: 2,
        blurb: "Arrows from afar, and traps for the packs that get close.",
    },
    ClassDef {
        id: 4,
        name: "Rogue",
        color: 0xFFF569,
        power: 3,
        start: "sinister_strike",
        pool: &[
            "sinister_strike",
            "eviscerate",
            "blade_flurry",
            "rupture",
            "gouge",
            "kidney_shot",
        ],
        looks: &[
            1297, 1327, 1328, 1507, 1580, 1603, 1704, 1725, 1886, 2231, 2243, 2252, 2631, 2639,
            2659, 3100, 3101, 3113, 3351, 3407, 3436, 3749, 4360, 5146, 5528, 13171,
        ],
        weapons: &[(0, 7492, 2, 7, 1, 13, 3), (1, 6443, 2, 15, 1, 13, 3)],
        sheath: 1,
        blurb: "Fast blades, bleeds and stuns.",
    },
    ClassDef {
        id: 5,
        name: "Priest",
        color: 0xFFFFFF,
        power: 0,
        start: "smite",
        pool: &[
            "smite",
            "shadow_word_pain",
            "holy_fire",
            "mind_blast",
            "holy_nova",
            "psychic_scream",
            "power_word_shield",
        ],
        looks: &[
            1295, 1495, 1579, 1602, 1708, 1733, 1897, 2137, 2138, 2139, 2200, 2201, 2202, 2618,
            2626, 3066, 3085, 3086, 3282, 3283, 3344, 3401, 3429, 4068, 4690, 4711, 10473, 10723,
            11044, 11048, 11053,
        ],
        weapons: &[(0, 1926, 2, 10, 2, 17, 2)],
        sheath: 1,
        blurb: "Holy light and shadow pain, with shields and healing.",
    },
    ClassDef {
        id: 7,
        name: "Shaman",
        color: 0x0070DE,
        power: 0,
        start: "lightning_bolt",
        pool: &[
            "lightning_bolt",
            "chain_lightning",
            "earth_shock",
            "flame_shock",
            "frost_shock",
            "searing_totem",
            "magma_totem",
            "lightning_shield",
        ],
        looks: &[
            1360, 1878, 2082, 2102, 2123, 3746, 3816, 4231, 4552, 10180, 13341,
        ],
        weapons: &[(0, 7477, 2, 4, 2, 13, 3), (1, 1705, 4, 6, 1, 14, 4)],
        sheath: 1,
        blurb: "Lightning, shocks and totems that fight beside you.",
    },
    ClassDef {
        id: 8,
        name: "Mage",
        color: 0x69CCF0,
        power: 0,
        start: "frostbolt",
        pool: &[
            "frostbolt",
            "fireball",
            "arcane_missiles",
            "frost_nova",
            "arcane_explosion",
            "blizzard",
            "flamestrike",
            "fire_blast",
            "cone_of_cold",
            "blast_wave",
            "pyroblast",
        ],
        looks: &[
            1294, 1470, 1484, 1592, 1600, 2134, 2135, 2644, 2657, 2810, 3108, 3109, 3292, 3293,
            4522, 4523, 4524, 4526, 4665, 5001, 6058, 6060, 6072, 7669, 10171, 10214, 10215, 10216,
            10548, 10733, 12849,
        ],
        weapons: &[(0, 1927, 2, 10, 2, 17, 2)],
        sheath: 1,
        blurb: "Fire, frost and arcane. Fragile, but the area damage is huge.",
    },
    ClassDef {
        id: 9,
        name: "Warlock",
        color: 0x9482C9,
        power: 0,
        start: "shadow_bolt",
        pool: &[
            "shadow_bolt",
            "corruption",
            "curse_of_agony",
            "immolate",
            "rain_of_fire",
            "hellfire",
            "death_coil",
            "searing_pain",
            "shadowburn",
        ],
        looks: &[
            1324, 1325, 1326, 1469, 1581, 1604, 1884, 1930, 2637, 2646, 2675, 3115, 3116, 3122,
            3271, 3286, 3291, 3345, 3607, 3745, 4567,
        ],
        weapons: &[(0, 5542, 2, 10, 2, 17, 2)],
        sheath: 1,
        blurb: "Curses and fire that grind the horde down over time.",
    },
    ClassDef {
        id: 11,
        name: "Druid",
        color: 0xFF7D0A,
        power: 0,
        start: "wrath",
        pool: &[
            "wrath",
            "moonfire",
            "starfire",
            "hurricane",
            "insect_swarm",
            "entangling_roots",
            "thorns",
            "rejuvenation",
        ],
        looks: &[
            1706, 1732, 2106, 2115, 2121, 2250, 2255, 2261, 3300, 3301, 3302, 3819, 3820, 7357,
            10738, 12053,
        ],
        weapons: &[(0, 22391, 2, 10, 2, 17, 2)],
        sheath: 1,
        blurb: "Nature's wrath, storms and roots, and healing to stay alive.",
    },
];

#[derive(Clone, Copy)]
pub(crate) struct EnemyDef {
    pub name: &'static str,
    pub display: u32,
    pub hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub scale: f32,
    pub xp: u32,
    pub level: u32,
    /// A great beast, which takes more room in the crowd.
    pub big: bool,
}

const fn en(
    name: &'static str,
    display: u32,
    hp: f32,
    speed: f32,
    damage: f32,
    xp: u32,
    level: u32,
) -> EnemyDef {
    EnemyDef {
        name,
        display,
        hp,
        speed,
        damage,
        scale: 1.0,
        xp,
        level,
        big: false,
    }
}

/// The Barrens, roughly in the order a character meets them.
pub(crate) static ENEMIES: &[EnemyDef] = &[
    en("Fleeting Plainstrider", 1284, 8.0, 3.6, 2.0, 1, 10),
    en("Zhevra Runner", 6087, 10.0, 4.4, 2.0, 1, 11),
    en("Razormane Quilboar", 1218, 16.0, 3.4, 3.0, 2, 12),
    en("Razormane Hunter", 6094, 18.0, 3.6, 4.0, 2, 12),
    en("Savannah Huntress", 1056, 22.0, 4.8, 8.0, 2, 13),
    en("Sunscale Lashtail", 1744, 26.0, 4.6, 8.0, 3, 13),
    en("Kolkar Wrangler", 9442, 36.0, 4.0, 10.0, 3, 14),
    en("Kolkar Stormer", 9443, 40.0, 4.0, 11.0, 3, 14),
    en("Witchwing Harpy", 3218, 34.0, 5.0, 10.0, 3, 15),
    en("Hecklefang Hyena", 2710, 44.0, 5.2, 12.0, 4, 15),
    en("Razormane Defender", 1253, 70.0, 3.3, 14.0, 5, 16),
    en("Razormane Mystic", 4643, 55.0, 3.8, 12.0, 4, 16),
    en("Sunscale Screecher", 1747, 60.0, 5.0, 13.0, 4, 17),
    en("Kolkar Bloodcharger", 9447, 85.0, 4.6, 16.0, 5, 18),
    en("Savannah Prowler", 1973, 75.0, 5.4, 15.0, 5, 18),
    en("Thunderhawk Hatchling", 1742, 90.0, 4.2, 16.0, 6, 19),
    en("Witchwing Slayer", 2163, 95.0, 5.0, 18.0, 6, 20),
    en("Bristleback Geomancer", 6091, 120.0, 3.6, 20.0, 7, 21),
    huge(en("Stormsnout", 1537, 150.0, 4.0, 22.0, 8, 22)),
    huge(en("Barrens Kodo", 1453, 220.0, 3.2, 28.0, 12, 23)),
    en("Sunscale Scytheclaw", 4442, 160.0, 5.6, 24.0, 8, 24),
    huge(en("Thunderhead", 1538, 180.0, 4.2, 26.0, 10, 25)),
    en("Silithid Swarmer", 2731, 90.0, 6.0, 18.0, 5, 25),
    huge(en("Greater Thunderhawk", 1974, 240.0, 4.4, 30.0, 12, 26)),
    en("Hecklefang Stalker", 2712, 200.0, 6.0, 30.0, 10, 27),
    en("Kolkar Destroyer", 9446, 280.0, 4.6, 34.0, 14, 28),
    en("Kolkar Mauler", 4874, 300.0, 4.6, 36.0, 14, 29),
    huge(en("Greater Barrens Kodo", 1232, 420.0, 3.4, 42.0, 24, 30)),
    huge(en("Elder Thunder Lizard", 2764, 380.0, 4.4, 44.0, 22, 31)),
    huge(en("Silithid Protector", 1307, 360.0, 5.0, 40.0, 20, 32)),
];

/// The great beasts at three quarters size, so a crowd of them still leaves the field readable.
const fn huge(def: EnemyDef) -> EnemyDef {
    EnemyDef {
        scale: 0.75,
        big: true,
        ..def
    }
}

pub(crate) static BOSSES: &[EnemyDef] = &[
    EnemyDef {
        scale: 1.6,
        ..en("Echeyakee", 1934, 700.0, 5.2, 20.0, 60, 16)
    },
    EnemyDef {
        scale: 1.6,
        ..en("Takk the Leaper", 1337, 1800.0, 5.4, 32.0, 120, 19)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Lakota'mani", 1241, 4200.0, 4.0, 45.0, 200, 22)
    },
    EnemyDef {
        scale: 1.7,
        ..en("Humar the Pridelord", 4424, 7000.0, 5.6, 55.0, 300, 26)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Agathelos the Raging", 2450, 12000.0, 5.0, 70.0, 400, 33)
    },
    EnemyDef {
        scale: 1.8,
        ..en("Hezrul Bloodmark", 9448, 20000.0, 4.8, 90.0, 500, 33)
    },
];

/// Survive this long to win.
pub(crate) const VICTORY_SECS: f32 = 900.0;
/// A boss every two and a quarter minutes, the last of six at 13:30, a fight before the bell.
pub(crate) const BOSS_EVERY_SECS: f32 = 135.0;
/// Experience per gem: every level of the curve comes at the run's pace.
pub(crate) const XP_RATE: f32 = 2.0;

/// How far through the run, 0 at the start and 1 at [`VICTORY_SECS`].
pub(crate) fn progress(elapsed: f32) -> f32 {
    (elapsed / VICTORY_SECS).max(0.0)
}

/// The pool of roster indices at a point in the run, from a roster `tiers` long.
pub(crate) fn wave_pool(elapsed: f32, tiers: usize) -> [usize; 3] {
    // The roster spread over thirty steps of the run, the last three from step 28: on The
    // Barrens' thirty tiers, one newer every step.
    let step = (progress(elapsed) * 30.0) as usize;
    let span = tiers.saturating_sub(3);
    let lo = (step.saturating_sub(1) * span / 27).min(span);
    [lo, lo + 1, lo + 2]
}

/// How many enemies the spawner keeps alive at a point in the run.
pub(crate) fn target_alive(elapsed: f32) -> usize {
    ((14.0 + progress(elapsed) * 225.0) as usize).min(220)
}

/// Enemy health grows on top of the tiers, so a late kill takes several spells.
pub(crate) fn hp_scale(elapsed: f32) -> f32 {
    1.0 + progress(elapsed) * 0.75
}

/// Enemy damage grows the same way.
pub(crate) fn damage_scale(elapsed: f32) -> f32 {
    1.0 + progress(elapsed) * 0.9
}

/// The experience to go from `level` to the next: Vampire Survivors' curve, cheap early.
pub(crate) fn xp_to_next(level: u32) -> f32 {
    let l = level as f32;
    if level < 20 {
        5.0 + (l - 1.0) * 10.0
    } else if level < 40 {
        195.0 + (l - 20.0) * 13.0
    } else {
        455.0 + (l - 40.0) * 16.0
    }
}

/// A creature on a battleground's roster: who it is, its pace and its size. Its health, damage
/// and experience come from its place on the roster, read off The Barrens' tuned ladder.
pub(crate) struct Foe {
    name: &'static str,
    display: u32,
    level: u32,
    speed: f32,
    /// The drawn scale, chosen against the model's Stand height so the field reads as The
    /// Barrens does: about 2-3.5 yd for the horde, 4-5 for the great beasts, 5-9 for a boss.
    scale: f32,
    big: bool,
}

const fn foe(name: &'static str, display: u32, level: u32, speed: f32, scale: f32) -> Foe {
    Foe {
        name,
        display,
        level,
        speed,
        scale,
        big: false,
    }
}

const fn giant(name: &'static str, display: u32, level: u32, speed: f32, scale: f32) -> Foe {
    Foe {
        big: true,
        ..foe(name, display, level, speed, scale)
    }
}

/// Where a run is fought.
pub(crate) struct MapDef {
    pub name: &'static str,
    /// The zone or continent under the name on the menu.
    pub zone: &'static str,
    /// The `Map.dbc` row.
    pub map: u32,
    /// Where the hero starts (WoW coordinates).
    pub center: [f32; 3],
    /// How far from the centre the hero may walk.
    pub leash: f32,
    /// The menu's line under the title.
    pub blurb: &'static str,
    /// The results screen's line on a win.
    pub victory: &'static str,
    /// Weakest first; empty for The Barrens, which keeps its hand-tuned [`ENEMIES`].
    foes: &'static [Foe],
    /// One every [`BOSS_EVERY_SECS`], in order; empty for The Barrens' [`BOSSES`].
    bosses: &'static [Foe],
}

pub(crate) static MAPS: &[MapDef] = &[
    MapDef {
        name: "The Barrens",
        zone: "Kalimdor",
        map: 1,
        // The plains south of the Crossroads.
        center: [-950.0, -2700.0, 92.0],
        leash: 150.0,
        blurb: "Survive fifteen minutes on the plains of The Barrens",
        victory: "The Barrens are yours.",
        foes: &[],
        bosses: &[],
    },
    MapDef {
        name: "The Dark Portal",
        zone: "Blasted Lands",
        map: 0,
        // The basin before the Portal, ringed by cliffs.
        center: [-11840.0, -3197.0, -30.0],
        leash: 75.0,
        blurb: "Hold the Blasted Lands before the Dark Portal",
        victory: "The Dark Portal stands silent.",
        foes: &[
            foe("Starving Snickerfang", 8050, 45, 4.6, 1.0),
            foe("Snickerfang Hyena", 2714, 49, 5.0, 1.15),
            foe("Black Slayer", 10824, 47, 5.0, 0.6),
            foe("Redstone Crystalhide", 798, 51, 3.6, 1.3),
            foe("Shadowsworn Cultist", 7838, 51, 3.8, 1.3),
            foe("Felbeast", 7949, 50, 4.8, 0.85),
            foe("Dreadmaul Mauler", 14401, 53, 3.6, 1.1),
            foe("Felhound", 1913, 54, 5.2, 1.0),
            foe("Felguard Sentry", 9017, 55, 4.2, 0.9),
            foe("Manahound", 6173, 60, 5.6, 1.15),
            giant("Lesser Infernal", 10906, 55, 3.4, 1.0),
            foe("Felguard Elite", 7970, 61, 4.4, 1.0),
            giant("Doomguard Commander", 4426, 61, 4.0, 1.1),
        ],
        bosses: &[
            foe("Akubar the Seer", 10920, 54, 4.4, 2.6),
            foe("Lord Azrethoc", 4426, 55, 4.6, 1.35),
            foe("Lord Hel'nurath", 14556, 62, 4.8, 1.65),
            giant("Teremus the Devourer", 6378, 60, 5.0, 0.55),
            foe("Lord Banehollow", 8611, 62, 4.8, 1.95),
            giant("Lord Kazzak", 12449, 63, 4.6, 0.75),
        ],
    },
    MapDef {
        name: "Gates of Ahn'Qiraj",
        zone: "Silithus",
        map: 1,
        // The sands before the Scarab Wall.
        center: [-8180.0, 1536.0, 5.0],
        leash: 80.0,
        blurb: "Stand before the Scarab Wall in Silithus",
        victory: "The Qiraji retreat behind the Scarab Wall.",
        foes: &[
            foe("Sand Skitterer", 8014, 56, 5.0, 1.0),
            foe("Stonelash Scorpid", 15383, 55, 4.6, 1.6),
            foe("Dredge Striker", 15386, 56, 4.4, 1.0),
            foe("Twilight Avenger", 11811, 58, 4.0, 1.1),
            foe("Hive'Ashi Drone", 2303, 58, 5.4, 0.55),
            foe("Stonelash Flayer", 15385, 57, 4.8, 1.75),
            foe("Hive'Ashi Defender", 12153, 59, 4.2, 1.3),
            foe("Dust Stormer", 8715, 57, 4.6, 1.05),
            foe("Twilight Stonecaller", 11815, 59, 3.8, 1.1),
            foe("Hive'Zora Wasp", 11088, 59, 5.6, 0.6),
            foe("Hive'Zora Reaver", 11085, 60, 4.8, 1.0),
            foe("Hive'Regal Ambusher", 11106, 60, 5.2, 1.05),
            foe("Vekniss Wasp", 15335, 60, 5.6, 0.65),
            foe("Qiraji Gladiator", 15741, 61, 4.4, 0.75),
            foe("Vekniss Warrior", 15334, 61, 4.6, 1.15),
            giant("Obsidian Destroyer", 15343, 61, 3.8, 1.2),
            giant("Qiraji Champion", 15340, 63, 4.6, 0.95),
            giant("Anubisath Sentinel", 15347, 61, 4.0, 0.62),
        ],
        bosses: &[
            giant("Kurinnaxx", 15742, 62, 5.0, 1.15),
            foe("General Rajaxx", 15376, 63, 4.8, 1.15),
            foe("Moam", 15392, 63, 4.2, 1.6),
            giant("Buru the Gorger", 15654, 63, 4.4, 0.85),
            giant("Princess Huhuran", 15739, 63, 5.0, 0.55),
            foe("Ossirian the Unscarred", 15432, 63, 4.6, 0.82),
        ],
    },
    MapDef {
        name: "Fire Plume Ridge",
        zone: "Un'Goro Crater",
        map: 1,
        // The summit plateau, its lava lake on the west rim; the crater floor's canopy would
        // hide the fight from above.
        center: [-7136.0, -1309.0, -184.0],
        leash: 32.0,
        blurb: "Hold the summit of Fire Plume Ridge above Un'Goro Crater",
        victory: "Even King Mosh has fallen.",
        foes: &[
            foe("Bloodpetal Lasher", 11634, 49, 3.6, 0.9),
            foe("Young Diemetradon", 8510, 48, 4.4, 0.55),
            foe("Ravasaur", 5242, 50, 5.4, 0.75),
            foe("Muculent Ooze", 11140, 50, 3.2, 1.4),
            foe("Ravasaur Hunter", 5292, 51, 5.6, 0.85),
            foe("Tar Beast", 1549, 51, 3.4, 1.0),
            foe("Bloodpetal Flayer", 11635, 52, 3.8, 1.05),
            foe("Primal Ooze", 4754, 52, 3.4, 1.6),
            foe("Gorishi Wasp", 11090, 53, 5.4, 0.55),
            foe("Pterrordax", 8411, 53, 5.2, 0.5),
            foe("Diemetradon", 8511, 52, 4.6, 0.7),
            foe("Glutinous Ooze", 1146, 54, 3.6, 1.8),
            foe("Tar Lurker", 9010, 54, 3.6, 1.15),
            foe("Scorching Elemental", 1070, 54, 4.2, 1.05),
            foe("Frenzied Pterrordax", 8412, 55, 5.6, 0.55),
            foe("Elder Diemetradon", 8512, 55, 4.6, 0.8),
            giant("Stegodon", 5241, 52, 3.8, 0.72),
            giant("Stone Guardian", 8395, 52, 3.6, 1.45),
            giant("Plated Stegodon", 5287, 54, 3.8, 0.78),
            giant("Devilsaur", 5239, 56, 5.0, 0.72),
        ],
        bosses: &[
            foe("Ravasaur Matriarch", 11319, 56, 5.6, 1.55),
            foe("Uhk'loc", 8129, 59, 4.8, 2.4),
            foe("Blazerunner", 1204, 57, 4.6, 1.8),
            foe("Baron Charr", 14517, 58, 4.6, 1.95),
            giant("Tyrant Devilsaur", 5240, 60, 5.0, 1.0),
            giant("King Mosh", 5305, 61, 5.0, 1.2),
        ],
    },
    MapDef {
        name: "Gurubashi Arena",
        zone: "Stranglethorn Vale",
        map: 0,
        // The arena floor; the stands ring it.
        center: [-13205.0, 272.0, 22.0],
        leash: 30.0,
        blurb: "Fight for the crowd in the Gurubashi Arena",
        victory: "The arena roars your name.",
        foes: &[
            foe("Shadowmaw Panther", 11452, 37, 5.6, 1.4),
            foe("Cold Eye Basilisk", 8797, 39, 3.8, 1.0),
            foe("Jungle Stalker", 11317, 40, 5.2, 0.8),
            foe("Elder Mistvale Gorilla", 838, 40, 4.0, 1.1),
            foe("Zanzil Zombie", 1065, 43, 3.4, 1.1),
            foe("Thrashtail Basilisk", 8802, 41, 4.0, 1.15),
            foe("Bloodsail Sea Dog", 796, 44, 4.0, 1.05),
            foe("Gurubashi Axe Thrower", 11074, 60, 4.2, 1.1),
            foe("Razzashi Serpent", 15182, 60, 4.8, 0.85),
            foe("Zulian Panther", 633, 60, 5.6, 1.6),
            foe("Gurubashi Headhunter", 11109, 60, 4.4, 1.15),
            foe("Bloodseeker Bat", 14562, 60, 5.6, 0.55),
            foe("Razzashi Raptor", 2571, 60, 5.6, 0.95),
            foe("Gurubashi Blood Drinker", 11080, 60, 4.2, 1.2),
            foe("Zulian Tiger", 11031, 61, 5.4, 1.8),
            foe("Hakkari Shadowcaster", 11229, 61, 4.0, 1.2),
            foe("Gurubashi Champion", 11100, 61, 4.4, 1.3),
            foe("Gurubashi Bat Rider", 15303, 62, 5.0, 0.6),
            giant("Gurubashi Berserker", 14832, 62, 4.0, 1.7),
        ],
        bosses: &[
            giant("King Bangalash", 616, 43, 5.8, 3.0),
            foe("Mokk the Savage", 840, 44, 4.8, 2.4),
            foe("Bloodlord Mandokir", 11288, 63, 5.0, 2.1),
            foe("High Priest Thekal", 15216, 63, 5.0, 2.3),
            foe("Jin'do the Hexxer", 11311, 63, 4.6, 2.5),
            giant("Hakkar", 15295, 63, 4.4, 2.7),
        ],
    },
    MapDef {
        name: "Blackrock Mountain",
        zone: "Burning Steppes",
        map: 0,
        // Blackrock Stronghold, under the burning peak.
        center: [-7733.0, -1510.0, 133.0],
        leash: 90.0,
        blurb: "Break the Blackrock legions beneath the burning peak",
        victory: "Blackrock Mountain falls silent.",
        foes: &[
            foe("Searing Lava Spider", 4457, 47, 4.6, 1.0),
            foe("Blackrock Worg", 741, 48, 5.4, 1.15),
            foe("Blazing Elemental", 1070, 49, 4.2, 1.0),
            foe("Scalding Broodling", 400, 50, 5.0, 1.3),
            foe("Thaurissan Spy", 6649, 50, 4.4, 1.1),
            foe("Greater Lava Spider", 7510, 51, 4.8, 1.15),
            foe("Blackrock Soldier", 6045, 52, 4.2, 1.1),
            foe("Flamekin Torcher", 10817, 53, 4.4, 1.0),
            foe("Blackrock Sorcerer", 6046, 53, 3.8, 1.1),
            foe("Giant Ember Worg", 9370, 54, 5.4, 1.5),
            foe("Flamescale Broodling", 457, 55, 5.2, 1.4),
            foe("Blackrock Slayer", 6047, 56, 4.6, 1.2),
            foe("Blackrock Warlock", 6048, 56, 3.8, 1.2),
            foe("Flamescale Dragonspawn", 2554, 57, 4.4, 1.25),
            giant("War Reaver", 10806, 57, 3.6, 1.4),
            foe("Blackrock Battlemaster", 6049, 58, 4.6, 1.35),
            giant("Blackrock Drake", 6374, 58, 5.0, 0.38),
            giant("Searscale Drake", 6377, 60, 5.0, 0.42),
        ],
        bosses: &[
            foe("Highlord Omokk", 11565, 60, 4.4, 1.8),
            giant("Volchan", 12232, 60, 3.8, 0.72),
            foe("Overlord Wyrmthalak", 8711, 63, 4.6, 1.8),
            foe("Emperor Dagran Thaurissan", 8807, 62, 4.6, 4.0),
            foe("General Drakkisath", 10115, 62, 4.6, 1.45),
            giant("Nefarian", 11380, 63, 4.8, 0.6),
        ],
    },
    MapDef {
        name: "Kodo Graveyard",
        zone: "Desolace",
        map: 1,
        // The bowl among the great bones.
        center: [-1273.0, 1971.0, 51.0],
        leash: 85.0,
        blurb: "Endure Desolace among the bones of the ancient kodo",
        victory: "The graveyard rests once more.",
        foes: &[
            foe("Scorpashi Snapper", 2729, 30, 4.4, 1.3),
            foe("Gritjaw Basilisk", 1074, 31, 3.8, 0.95),
            foe("Dread Swoop", 1192, 32, 5.2, 0.55),
            foe("Scorpashi Lasher", 2765, 34, 4.6, 1.5),
            foe("Rabid Bonepaw", 10271, 35, 5.4, 1.15),
            foe("Hulking Gritjaw Basilisk", 7345, 36, 3.8, 1.25),
            foe("Whirlwind Stormwalker", 11373, 36, 4.8, 0.95),
            foe("Carrion Horror", 10825, 37, 5.2, 0.6),
            foe("Burning Blade Summoner", 4709, 37, 3.8, 1.1),
            foe("Nether Maiden", 159, 38, 4.6, 1.3),
            foe("Ghost Walker Brave", 7374, 39, 4.0, 1.2),
            foe("Mana Eater", 1913, 39, 5.2, 1.0),
            foe("Dread Flyer", 10273, 40, 5.4, 0.62),
            foe("Doomwarder", 5049, 40, 4.2, 0.95),
            giant("Lesser Infernal", 10906, 40, 3.4, 1.0),
            giant("Aged Kodo", 1308, 34, 3.4, 0.8),
            giant("Dying Kodo", 1453, 40, 3.2, 0.85),
            giant("Ancient Kodo", 2767, 42, 3.4, 0.9),
            giant("Elder Thunder Lizard", 2764, 42, 4.4, 0.8),
        ],
        bosses: &[
            giant("Hissperak", 2076, 37, 4.6, 2.3),
            foe("Lord Vyletongue", 12334, 47, 5.2, 2.0),
            foe("Noxxion", 11172, 48, 4.4, 2.25),
            foe("Celebras the Cursed", 12350, 49, 4.4, 1.55),
            giant("Landslide", 12293, 50, 4.0, 0.92),
            giant("Princess Theradras", 12292, 51, 4.6, 1.95),
        ],
    },
    MapDef {
        name: "Winterspring",
        zone: "Lake Kel'Theril",
        map: 1,
        // The frozen shore of Lake Kel'Theril, clear of the snow pines to the north.
        center: [6425.0, -4295.0, 663.0],
        leash: 45.0,
        blurb: "Brave the frozen shores of Lake Kel'Theril",
        victory: "Winterspring thaws in your wake.",
        foes: &[
            foe("Winterspring Owl", 6212, 54, 5.2, 0.5),
            foe("Shardtooth Bear", 865, 54, 4.6, 1.3),
            foe("Fledgling Chillwind", 10807, 54, 5.4, 0.6),
            foe("Rogue Ice Thistle", 6767, 55, 3.8, 0.85),
            foe("Ragged Owlbeast", 12235, 55, 4.4, 0.7),
            foe("Winterfall Runner", 6829, 56, 5.2, 1.5),
            foe("Cobalt Whelp", 10585, 56, 5.0, 1.3),
            foe("Suffering Highborne", 10701, 56, 3.6, 0.9),
            foe("Spell Eater", 10583, 57, 5.0, 1.4),
            foe("Shardtooth Mauler", 8842, 57, 4.8, 1.5),
            foe("Hederine Initiate", 10924, 57, 4.6, 1.5),
            foe("Cobalt Broodling", 10584, 58, 5.0, 1.55),
            foe("Hederine Manastalker", 6172, 58, 5.4, 1.15),
            foe("Cobalt Wyrmkin", 6761, 59, 4.2, 1.35),
            foe("Chillwind Chimaera", 10810, 59, 5.4, 0.75),
            foe("Rabid Shardtooth", 3200, 59, 5.0, 1.75),
            foe("Watery Invader", 5489, 59, 4.0, 1.35),
            foe("Hederine Slayer", 9018, 60, 4.4, 1.0),
            giant("Frostmaul Giant", 6209, 60, 3.6, 0.62),
        ],
        bosses: &[
            giant("Rak'shiri", 10054, 57, 6.0, 3.0),
            giant("Spellmaw", 9995, 57, 5.0, 0.45),
            foe("General Colbatann", 9489, 61, 4.6, 1.8),
            giant("Kashoch the Reaver", 10317, 60, 3.8, 0.82),
            foe("Princess Tempestria", 14514, 60, 4.4, 2.6),
            giant("Azuregos", 11460, 63, 4.8, 0.6),
        ],
    },
    MapDef {
        name: "Mount Hyjal",
        zone: "Kalimdor",
        map: 1,
        // A plateau below the World Tree.
        center: [5478.0, -3729.0, 1593.0],
        leash: 70.0,
        blurb: "Defend the slopes of Mount Hyjal beneath the World Tree",
        victory: "Nordrassil endures.",
        foes: &[
            foe("Cannibal Ghoul", 10626, 56, 4.6, 1.3),
            foe("Jadefire Rogue", 8575, 52, 5.2, 1.0),
            foe("Putrid Gargoyle", 7854, 56, 5.2, 0.85),
            foe("Death Cultist", 10391, 56, 3.8, 1.1),
            foe("Felbeast", 7949, 50, 4.8, 0.85),
            foe("Torn Screamer", 984, 57, 5.0, 0.7),
            foe("Necromancer", 10427, 57, 3.6, 1.15),
            foe("Jadefire Felsworn", 11334, 54, 4.8, 1.1),
            foe("Felhound", 1913, 54, 5.2, 1.0),
            foe("Nerubian Overseer", 14698, 58, 4.4, 0.95),
            foe("Felguard Sentry", 9017, 55, 4.2, 0.9),
            foe("Hate Shrieker", 10751, 58, 5.0, 0.8),
            foe("Hederine Manastalker", 6172, 58, 5.4, 1.15),
            giant("Lesser Infernal", 10906, 55, 3.4, 1.0),
            foe("Felguard Elite", 7970, 61, 4.4, 1.0),
            giant("Stitched Horror", 1693, 58, 3.6, 1.25),
            giant("Doomguard Commander", 4426, 61, 4.0, 1.1),
        ],
        bosses: &[
            foe("Prince Xavalis", 11335, 57, 5.0, 1.85),
            foe("Lady Hederine", 10925, 61, 4.8, 2.7),
            giant("Taerar", 15363, 63, 4.8, 0.5),
            giant("Lethon", 15365, 63, 4.8, 0.5),
            giant("Emeriss", 15366, 63, 4.8, 0.53),
            giant("Ysondre", 15364, 63, 4.8, 0.58),
        ],
    },
    MapDef {
        name: "Naxxramas",
        zone: "Eastern Plaguelands",
        map: 0,
        // Plaguewood, under the necropolis.
        center: [3010.0, -3790.0, 124.0],
        leash: 80.0,
        blurb: "Face the Scourge in the shadow of Naxxramas",
        victory: "Kel'Thuzad's necropolis falls dark.",
        foes: &[
            foe("Scourge Soldier", 5231, 55, 4.4, 1.15),
            foe("Plagued Swine", 6121, 53, 5.0, 1.0),
            foe("Cannibal Ghoul", 10626, 56, 4.6, 1.3),
            foe("Cursed Mage", 11397, 55, 3.6, 0.95),
            foe("Putrid Gargoyle", 7854, 56, 5.2, 0.85),
            foe("Scourge Warder", 612, 56, 4.4, 1.2),
            foe("Death Cultist", 10391, 56, 3.8, 1.1),
            foe("Frenzied Plaguehound", 9022, 57, 5.6, 1.4),
            foe("Torn Screamer", 984, 57, 5.0, 0.7),
            foe("Necromancer", 10427, 57, 3.6, 1.15),
            foe("Carrion Devourer", 7900, 57, 4.0, 1.25),
            foe("Crypt Walker", 11178, 58, 4.6, 0.85),
            foe("Putrid Shrieker", 7855, 58, 5.4, 1.0),
            foe("Hate Shrieker", 10751, 58, 5.0, 0.8),
            foe("Ziggurat Protector", 10627, 59, 4.4, 1.6),
            foe("Deathknight", 16508, 60, 4.6, 1.75),
            giant("Stitched Horror", 1693, 58, 3.6, 1.25),
            giant("Slaughterhouse Protector", 7864, 60, 3.6, 1.3),
            giant("Death Lord", 10729, 61, 5.0, 1.05),
        ],
        bosses: &[
            giant("Anub'Rekhan", 15931, 63, 4.6, 0.55),
            foe("Grand Widow Faerlina", 15940, 63, 4.8, 2.9),
            giant("Patchwerk", 16174, 63, 4.0, 1.7),
            giant("Gluth", 16064, 63, 5.0, 0.62),
            giant("Thaddius", 16137, 63, 4.0, 0.98),
            foe("Kel'Thuzad", 15945, 63, 4.4, 2.1),
        ],
    },
];

/// A battleground's horde and bosses with their numbers.
pub(crate) struct Roster {
    pub enemies: Vec<EnemyDef>,
    pub bosses: Vec<EnemyDef>,
}

static ROSTERS: std::sync::LazyLock<Vec<Roster>> = std::sync::LazyLock::new(|| {
    MAPS.iter()
        .map(|m| {
            if m.foes.is_empty() {
                Roster {
                    enemies: ENEMIES.to_vec(),
                    bosses: BOSSES.to_vec(),
                }
            } else {
                Roster {
                    enemies: ladder(m.foes, ENEMIES),
                    bosses: ladder(m.bosses, BOSSES),
                }
            }
        })
        .collect()
});

pub(crate) fn roster(map: usize) -> &'static Roster {
    &ROSTERS[map.min(ROSTERS.len() - 1)]
}

/// Number a roster off a tuned ladder: each creature takes the health, damage and experience at
/// its own place along it, from the ladder's running peak so no later tier comes out softer.
fn ladder(foes: &[Foe], tuned: &[EnemyDef]) -> Vec<EnemyDef> {
    let mut peak = (0.0f32, 0.0f32, 0.0f32);
    let rungs: Vec<(f32, f32, f32)> = tuned
        .iter()
        .map(|d| {
            peak = (
                peak.0.max(d.hp),
                peak.1.max(d.damage),
                peak.2.max(d.xp as f32),
            );
            peak
        })
        .collect();
    let top = (rungs.len() - 1) as f32;
    foes.iter()
        .enumerate()
        .map(|(i, f)| {
            let t = if foes.len() > 1 {
                i as f32 / (foes.len() - 1) as f32 * top
            } else {
                0.0
            };
            let (lo, hi) = (rungs[t.floor() as usize], rungs[t.ceil() as usize]);
            let k = t.fract();
            let mix = |a: f32, b: f32| (a + (b - a) * k).round();
            EnemyDef {
                name: f.name,
                display: f.display,
                hp: mix(lo.0, hi.0),
                speed: f.speed,
                damage: mix(lo.1, hi.1),
                scale: f.scale,
                xp: mix(lo.2, hi.2) as u32,
                level: f.level,
                big: f.big,
            }
        })
        .collect()
}
