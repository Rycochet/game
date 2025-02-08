/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

// https://www.rpgdl.com/forums/index.php?topic=1744.0

export enum Skill {
    OFFENSE = "offense",
    OFFENSE_BLUDGEON = "offense.bludgeon",
    OFFENSE_SLICE = "offense.slice",
    OFFENSE_STAB = "offense.stab",
    DEFENSE = "defense",
    MAGIC = "magic",

    /** Directly ahead / continue last missile direction. */
    MAGIC_TARGET_AHEAD = "magic.target.ahead",
    /** Flies in direction, impacts first hit target only. */
    MAGIC_TARGET_MISSILE = "magic.target.missile",
    /** Hits all targets in direction. */
    MAGIC_TARGET_BEAM = "magic.target.beam",
    /** Surrounds in all 8 squares, direction from centre. */
    MAGIC_TARGET_SURROUND = "magic.target.surround",
    /** Create a right angle wall. */
    MAGIC_TARGET_WALL = "magic.target.wall",

    MAGIC_ELEMENT_EARTH = "magic.element.earth",
    MAGIC_ELEMENT_AIR = "magic.element.air",
    MAGIC_ELEMENT_FIRE = "magic.element.fire",
    MAGIC_ELEMENT_WATER = "magic.element.water",
    MAGIC_ELEMENT_AETHER = "magic.element.aether",

    /**
     * Adds a permanent affect to the target. Duration depends on skill. If it
     * is on an entity when it fires that entity is affected, if on space then
     * a (shorter duration) affect will apply.
     *
     * - MAGIC_ELEMENT_EARTH: Paralysis
     * - MAGIC_ELEMENT_AIR: Flight
     * - MAGIC_ELEMENT_FIRE: Blind
     * - MAGIC_ELEMENT_WATER: Water breathing
     * - MAGIC_ELEMENT_AETHER: Confusion
     *
     * - MAGIC_EFFECT_DAMAGE: Damage over time
     * - MAGIC_EFFECT_REPAIR: Regeneration
     */
    MAGIC_EFFECT_PERMANENT = "magic.effect.permanent",

    /** Damage, if no element then is plain "force", but easiest to block. */
    MAGIC_EFFECT_DAMAGE = "magic.effect.damage",

    /**
     * With elements can do more powerful healing by targetting weaknesses.
     *
     * - MAGIC_ELEMENT_EARTH: Cure Paralysis
     * - MAGIC_ELEMENT_AIR: Flight
     * - MAGIC_ELEMENT_FIRE: Cure Blindness
     * - MAGIC_ELEMENT_WATER: Water breathing
     * - MAGIC_ELEMENT_AETHER: Confusion
     */
    MAGIC_EFFECT_REPAIR = "magic.effect.repair",

    /**
     * Elements make it more powerful against specific effects.
     * Earth: PARALYSIS
     * Air: FLIGHT
     * Fire:
     * Water: REGENERATION
     */
    MAGIC_EFFECT_DISPEL = "magic.effect.dispel",

    /** Elements can help different things. */
    MAGIC_EFFECT_SPEED = "magic.effect.speed",


    /**
    Damage - All-purpose Damage spell. Non Elemental.
    Heal - Restores health.
    Dispel - Removes status.
    Speed - Applies Speed status.
    Paralyze - Applies Paralyze status.
    Antimage - Applies Antimage status.
    Thrall - Applies Antimage status.
    Make Weapon - Creates a magical weapon in inventory and equips it.
    Teleport - Applies Teleport status.
    Regeneration - Applies Regeneration status.
    Disrupt - Super-Damage. Hits through Antimage status, but is weakened. Non Elemental. Bosses ingame are immune to it.
    Vivify - Undoes Dead status, grants a minor amount of Luck (only to people who had Dead status).
     */
}

export type Skills = Record<Skill, number>;
export type SkillList = ReadonlyArray<Skill>;

/**
 * Used to split a skill into value & experience. Max skill is 100^3, so 1m.
 */
export const SKILL_SCALE = 1000000;

export interface Entity {
    name: string;
    id: string;
    skills: Skills;
}

export interface Spell {
    skills: Skills;
}

export interface Item {
    skills: Skills;
}

/**
 * Check the skill use of an Entity, returns the average of all the skills used,
 * so if this uses more skills it might push the value up or down depending on
 * what is needed.
 *
 * Every skill used can optionally increase in experience.
 *
 * NOTE: If more skills are used then it should reward that detail, and not make
 * the simple skills the preferred skills.
 */
export function useSkill(entity: Entity, skills: SkillList, exp = false) {
    let total = 0;

    for (const skill of skills) {
        const current = entity.skills[skill];
        const value = current / SKILL_SCALE;

        total += Math.cbrt(value);

        if (exp && value < 100) {
            entity.skills[skill] = (current % SKILL_SCALE) + 1 >= Math.pow(value, 3)
                ? (value + 1) * SKILL_SCALE
                : current + 1;
        }
    }

    return total / skills.length;
}
