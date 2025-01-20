/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

// Based on https://github.com/coverslide/node-alea

export const ALEA_VERSION = "Alea 0.10";

/**
 * Mash any stringable value into a single 32-bit number.
 */
function Mash() {
    let n = 0xefc8249d;

    return (value: { toString: () => string }) => {
        const data = value.toString();

        for (let h, i = 0; i < data.length; i++) {
            n += data.charCodeAt(i);
            h = 0.02519603282416938 * n;
            n = h >>> 0;
            h -= n;
            h *= n;
            n = h >>> 0;
            h -= n;
            n += h * 0x100000000; // 2^32
        }

        return (n >>> 0) * 2.3283064365386963e-10; // 2^-32
    };
}

/**
 * A sequence of 4 numbers, and a version string so it can be imported and
 * exported safely.
 */
export type AleaState = [number, number, number, number] & { version: string };

/**
 * The return value from the Alea() constuctor.
 */
export interface Alea {
    /**
     * Used to track the version, should match `ALEA_VERSION`.
     */
    version: string;

    /**
     * Call as a function to get the next number.
     *
     * @returns -1 < n < 1
     */
    (): number;

    /**
     * Alias for calling directly.
     *
     * @returns -1 < n < 1
     */
    next: () => number;

    /**
     * Gives a boolean.
     */
    bool: () => boolean;

    /**
     * Get a signed number (non-integer).
     *
     * @param min 0
     * @param max 1
     *
     * @returns min < n < max
     */
    abs: (max?: number, min?: number) => number;

    /**
     * Get a signed number within range (non-integer).
     *
     * @param max 0x100000000 (2^32)
     * @param min -max
     *
     * @returns min < n < max
     */
    frac: (max?: number, min?: number) => number;

    /**
     * Get an unsigned number within range (non-integer).
     *
     * @param max 0x100000000 (2^32)
     * @param min -max
     *
     * @returns min < n < max
     */
    ufrac: (max?: number, min?: number) => number;

    /**
     * Get a signed integer.
     *
     * @param max 0x100000000 (2^32)
     *
     * @returns -max < n < max
     */
    int: (max?: number) => number;

    /**
     * Get an unsigned integer.
     *
     * @param min 0
     * @param max 0x100000000 (2^32)
     *
     * @returns min < n < max
     */
    uint: (max?: number, min?: number) => number;

    /**
     * Get a valid fraction.
     */
    frac53: () => number;

    /**
     * Export the current Alea state so the pattern can be duplicated.
     */
    exportState: () => AleaState;

    /**
     * Import an Alea state.
     */
    importState: (state: AleaState) => Alea;
}

/**
 * Random number generator which can take a list of seed values and produce the
 * same consistent pseudo-random pattern.
 *
 * @param args One or more values to be used as seed, or an Alea instance, or
 *             an exported Alea state.
 */
export const Alea = (...args: any[]): Alea => {
    let s0 = 0;
    let s1 = 0;
    let s2 = 0;
    let c = 1;

    const rng = (() => {
        const t = 2091639 * s0 + c * 2.3283064365386963e-10;

        return (s0 = s1), (s1 = s2), (s2 = t - (c = t | 0));
    }) as any as Alea;

    Object.defineProperties(rng, {
        version: { value: ALEA_VERSION },
        next: { value: rng },
        bool: { value: () => ((rng() * 10) & 1) === 1 },
        abs: { value: (max = 1, min = 0) => Math.abs(rng() * (max - min)) + min },
        int: { value: (max = 0x100000000, min = -max) => Math.round(rng() * (max - min) + min) },
        uint: { value: (max = 0x100000000, min = 0) => Math.floor(Math.abs(rng()) * (max - min) + min) },
        frac: { value: (max = 0x100000000, min = -max) => rng() * (max - min) + min },
        ufrac: { value: (max = 0x100000000, min = 0) => Math.abs(rng() * (max - min)) + min },
        fract53: { value: () => rng() + ((rng() * 0x200000) | 0) * 1.1102230246251565e-16 }, // 2^-53
        exportState: {
            value: () => Object.defineProperty([s0, s1, s2, c] as AleaState, "version", { value: ALEA_VERSION }),
        },
        importState: {
            value: (state: AleaState) => {
                if (state.version === ALEA_VERSION) {
                    s0 = +state[0] || 0;
                    s1 = +state[1] || 0;
                    s2 = +state[2] || 0;
                    c = +state[3] || 0;
                }

                return rng;
            },
        },
    });

    if (args.length === 1 && args[0].version === ALEA_VERSION) {
        if (args[0].length === 4) {
            rng.importState(args[0]);
        } else {
            rng.importState(args[0].exportState());
        }
    } else {
        const mash = Mash();

        s0 = mash(" ");
        s1 = mash(" ");
        s2 = mash(" ");

        if (!args.length) {
            args = [+new Date()];
        }

        for (const arg of args) {
            s0 -= mash(arg);
            s0 += s0 < 0 ? 1 : 0;
            s1 -= mash(arg);
            s1 += s1 < 0 ? 1 : 0;
            s2 -= mash(arg);
            s2 += s2 < 0 ? 1 : 0;
        }
    }

    return rng;
};
