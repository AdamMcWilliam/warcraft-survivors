//! Local units and effects, built the way the net layer builds streamed ones: a [`Guid`], a
//! [`NetEntity`] and an [`ObjectStore`] are all the attach, animation and spell-visual systems need.

use benilla_protocol::messages::{ObjectFields, ObjectType};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::data::Weapon;
use super::RunEntity;
use crate::combat_text::CombatTextSpawn;
use crate::creature_anim::{CastEvent, CastEventKind, PlaySeq, SpellGoTargets, SwingMessage};
use crate::net::{FieldChanged, Guid, NetEntity, ObjectStore};

const FIELD_HEALTH: u16 = 22;
const FIELD_MAX_HEALTH: u16 = 28;
const FIELD_LEVEL: u16 = 34;
const FIELD_FACTION: u16 = 35;
const FIELD_BYTES_0: u16 = 36;
const FIELD_VIRTUAL_ITEM_DISPLAY: u16 = 37;
const FIELD_VIRTUAL_ITEM_INFO: u16 = 40;
const FIELD_AURA: u16 = 47;
const FIELD_AURAFLAGS: u16 = 95;
const FIELD_BYTES_2: u16 = 164;

/// `FactionTemplate.dbc`: friendly to players, and the hostile monster template.
pub(super) const FACTION_FRIENDLY: u32 = 35;
pub(super) const FACTION_MONSTER: u32 = 14;

const HIGH_CREATURE: u64 = 0xF130 << 48;
const HIGH_GAMEOBJECT: u64 = 0xF110 << 48;
const HIGH_DYNAMIC: u64 = 0xF100 << 48;

const FIELD_GO_DISPLAY: u16 = 8;
const FIELD_GO_STATE: u16 = 14;
const FIELD_GO_TYPE: u16 = 21;
/// `GAMEOBJECT_TYPE_GENERIC`: a model and nothing to use.
const GO_TYPE_GENERIC: u32 = 5;
/// `GO_STATE_READY`.
const GO_STATE_READY: u32 = 1;

pub(super) fn wow_to_bevy(p: [f32; 3]) -> Vec3 {
    Vec3::new(-p[1], p[2], -p[0])
}

fn bevy_to_wow(p: Vec3) -> [f32; 3] {
    [-p.z, -p.x, p.y]
}

/// The Bevy yaw that faces a model along `dir` (its XZ part).
pub(super) fn yaw_toward(dir: Vec3) -> f32 {
    (-dir.x).atan2(-dir.z)
}

pub(super) fn flat(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

pub(super) struct UnitSpec<'a> {
    pub serial: u64,
    pub display: u32,
    pub scale: f32,
    pub level: u32,
    pub faction: u32,
    /// `UNIT_FIELD_BYTES_0`: race, class, gender, power type.
    pub bytes0: u32,
    pub weapons: &'a [Weapon],
    /// `UNIT_FIELD_BYTES_2` byte 0: 0 stowed, 1 melee drawn, 2 ranged drawn.
    pub sheath: u8,
}

/// Spawn a unit the attach builds a body for; the caller adds its own components.
pub(super) fn spawn_unit(commands: &mut Commands, spec: &UnitSpec, pos: Vec3, yaw: f32) -> Entity {
    let guid = HIGH_CREATURE | spec.serial;
    let mut pairs: Vec<(u16, u32)> = vec![
        (0, guid as u32),
        (1, (guid >> 32) as u32),
        (2, 0x9),
        (4, spec.scale.to_bits()),
        (FIELD_HEALTH, 100),
        (FIELD_MAX_HEALTH, 100),
        (FIELD_LEVEL, spec.level),
        (FIELD_FACTION, spec.faction),
        (FIELD_BYTES_0, spec.bytes0),
        (FIELD_BYTES_2, u32::from(spec.sheath)),
    ];
    for &(slot, display, class, sub, material, inv, sheath) in spec.weapons {
        pairs.push((FIELD_VIRTUAL_ITEM_DISPLAY + slot, display));
        pairs.push((
            FIELD_VIRTUAL_ITEM_INFO + 2 * slot,
            u32::from(class)
                | u32::from(sub) << 8
                | u32::from(material) << 16
                | u32::from(inv) << 24,
        ));
        pairs.push((FIELD_VIRTUAL_ITEM_INFO + 2 * slot + 1, u32::from(sheath)));
    }
    commands
        .spawn((
            Guid(guid),
            NetEntity {
                kind: benilla_protocol::EntityKind::Unit,
                display_id: Some(spec.display),
                scale: spec.scale,
            },
            ObjectStore(ObjectFields::from_pairs(&pairs)),
            crate::creature_anim::MovementState::default(),
            crate::net::ScriptedFacing,
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(yaw)),
            Visibility::default(),
            RunEntity,
        ))
        .id()
}

/// Spawn a generic GameObject showing `GameObjectDisplayInfo` row `display`.
pub(super) fn spawn_object(
    commands: &mut Commands,
    serial: u64,
    display: u32,
    scale: f32,
    pos: Vec3,
) -> Entity {
    let guid = HIGH_GAMEOBJECT | serial;
    let pairs = [
        (0, guid as u32),
        (1, (guid >> 32) as u32),
        (2, 0x21),
        (4, scale.to_bits()),
        (FIELD_GO_DISPLAY, display),
        (FIELD_GO_STATE, GO_STATE_READY),
        (FIELD_GO_TYPE, GO_TYPE_GENERIC),
    ];
    commands
        .spawn((
            Guid(guid),
            NetEntity {
                kind: benilla_protocol::EntityKind::GameObject,
                display_id: Some(display),
                scale,
            },
            ObjectStore(ObjectFields::from_pairs(&pairs)),
            Transform::from_translation(pos),
            Visibility::default(),
            RunEntity,
        ))
        .id()
}

/// Zero health: the animation selector plays Death and holds the corpse pose.
pub(super) fn kill_unit(store: &mut ObjectStore) {
    store
        .0
        .merge(ObjectFields::from_pairs(&[(FIELD_HEALTH, 0)]));
}

/// A dynamic object, as a ground spell's server would place it: the dest-effect systems arm the
/// spell's area visuals on it.
pub(super) fn spawn_area_object(
    commands: &mut Commands,
    serial: u64,
    spell: u32,
    radius: f32,
    pos: Vec3,
) -> Entity {
    let guid = HIGH_DYNAMIC | serial;
    let wow = bevy_to_wow(pos);
    commands
        .spawn((
            Guid(guid),
            NetEntity {
                kind: benilla_protocol::EntityKind::DynamicObject,
                display_id: None,
                scale: 1.0,
            },
            ObjectStore(ObjectFields::from_pairs(&[
                (0, guid as u32),
                (1, (guid >> 32) as u32),
                (2, 0x41),
                (3, spell),
                (4, 1.0f32.to_bits()),
                (8, 1),
                (9, spell),
                (10, radius.to_bits()),
                (11, wow[0].to_bits()),
                (12, wow[1].to_bits()),
                (13, wow[2].to_bits()),
                (14, 0),
            ])),
            Transform::from_translation(pos),
            Visibility::default(),
            RunEntity,
        ))
        .id()
}

/// Eight aura slots we drive on a unit, from `base` (a multiple of 8, so one flag word covers
/// them): the spell visuals' state kits arm and reap off these as off a server's.
#[derive(Component)]
pub(super) struct Marks {
    base: u8,
    slots: [(u32, f32); 8],
    dirty: bool,
}

impl Marks {
    pub fn buffs() -> Self {
        Self {
            base: 0,
            slots: [(0, 0.0); 8],
            dirty: false,
        }
    }
    pub fn debuffs() -> Self {
        Self {
            base: 32,
            slots: [(0, 0.0); 8],
            dirty: false,
        }
    }

    /// Show `spell` for `secs`, refreshing it if already shown.
    pub fn add(&mut self, spell: u32, secs: f32) {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.0 == spell) {
            slot.1 = slot.1.max(secs);
            return;
        }
        if let Some(slot) = self.slots.iter_mut().find(|s| s.0 == 0) {
            *slot = (spell, secs);
            self.dirty = true;
        }
    }

    /// Exactly these spells, held until changed.
    pub fn set(&mut self, spells: &[u32]) {
        let mut want = [(0u32, f32::INFINITY); 8];
        for (slot, &spell) in want.iter_mut().zip(spells) {
            slot.0 = spell;
        }
        if want.iter().map(|s| s.0).ne(self.slots.iter().map(|s| s.0)) {
            self.slots = want;
            self.dirty = true;
        }
    }

    pub fn clear(&mut self) {
        if self.slots.iter().any(|s| s.0 != 0) {
            self.slots = [(0, 0.0); 8];
            self.dirty = true;
        }
    }

    pub fn tick(&mut self, dt: f32) {
        for slot in &mut self.slots {
            if slot.0 != 0 {
                slot.1 -= dt;
                if slot.1 <= 0.0 {
                    *slot = (0, 0.0);
                    self.dirty = true;
                }
            }
        }
    }

    /// Write the slots into the unit's aura arrays and ring the watch, if they changed.
    pub fn flush(
        &mut self,
        entity: Entity,
        guid: u64,
        store: &mut ObjectStore,
        fields: &mut MessageWriter<FieldChanged>,
    ) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        let mut pairs = Vec::with_capacity(9);
        let mut word = 0u32;
        for (i, &(spell, _)) in self.slots.iter().enumerate() {
            pairs.push((FIELD_AURA + u16::from(self.base) + i as u16, spell));
            if spell != 0 {
                // Effect 0 live, and a buff is cancelable.
                let flags = 0x2 | u32::from(self.base < 32);
                word |= flags << (i * 4);
            }
        }
        pairs.push((FIELD_AURAFLAGS + u16::from(self.base >> 3), word));
        store.0.merge(ObjectFields::from_pairs(&pairs));
        fields.write(FieldChanged {
            entity,
            guid,
            kind: ObjectType::Unit,
            index: FIELD_AURA + u16::from(self.base),
            old: 0,
            new: 1,
        });
    }
}

/// The messages that make a local cast look like a server's.
#[derive(SystemParam)]
pub(super) struct Fx<'w> {
    seq: ResMut<'w, PlaySeq>,
    casts: MessageWriter<'w, CastEvent>,
    gos: MessageWriter<'w, SpellGoTargets>,
    swings: MessageWriter<'w, SwingMessage>,
    text: MessageWriter<'w, CombatTextSpawn>,
}

/// More impacts than this per cast only cost frames.
const MAX_VISUAL_HITS: usize = 12;

impl Fx<'_> {
    /// `SMSG_SPELL_GO`: the cast kit on the caster, and a missile or an impact at each hit.
    pub fn cast(&mut self, caster: Entity, spell: u32, hits: &[Entity], dest: Option<Vec3>) {
        let seq = self.seq.next();
        self.casts.write(CastEvent {
            entity: caster,
            spell_id: spell,
            kind: CastEventKind::Go,
            seq,
        });
        self.gos.write(SpellGoTargets {
            caster,
            spell_id: spell,
            hits: hits.iter().copied().take(MAX_VISUAL_HITS).collect(),
            misses: Vec::new(),
            dest,
            ammo_display_id: None,
            seq,
        });
    }

    /// Only the spell's impact kit, on `target`: no cast animation on anyone.
    pub fn impact(&mut self, target: Entity, spell: u32) {
        let seq = self.seq.next();
        self.casts.write(CastEvent {
            entity: target,
            spell_id: spell,
            kind: CastEventKind::Impact {
                weapon_visual: None,
            },
            seq,
        });
    }

    /// A melee swing: the attack animation, the victim's flinch, blood and the hit sound.
    pub fn swing(&mut self, attacker: Entity, victim: Entity, damage: f32) {
        let seq = self.seq.next();
        self.swings.write(SwingMessage {
            attacker,
            victim: Some(victim),
            hit_info: 0x2,
            victim_state: 1,
            damage: damage.max(0.0) as u32,
            displayed: false,
            seq,
        });
    }

    /// A floating number over `anchor`, 0xAARRGGBB.
    pub fn number(&mut self, anchor: Entity, amount: f32, crit: bool, color: u32) {
        let shown = amount.round().max(1.0) as i64;
        self.text.write(CombatTextSpawn {
            anchor,
            text: shown.to_string(),
            category: if crit { 2 } else { 0 },
            color: Some(color),
        });
    }
}

pub(super) const COLOR_SPELL: u32 = 0xFFFF_DE00;
pub(super) const COLOR_MELEE: u32 = 0xFFFF_FFFF;
pub(super) const COLOR_HURT: u32 = 0xFFFF_3030;
pub(super) const COLOR_HEAL: u32 = 0xFF30_FF30;
