/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import type { NoiseFunction3D } from "simplex-noise";
import { findWeakKey, freeWeakKey, getWeakKey } from "../hooks";

const TAU = 2 * Math.PI; // Radius ratio to world size

/**
 * An individual chunk of world surface.
 */
export interface Chunk {
    x: number;
    y: number;

    /**
     * The physical height of this point.
     */
    height: Array<number>;

    /**
     * The virtual relative height - used for terrain smoothing.
     */
    jitter: Array<number>;
}

/**
 * A Surface is used to dynamically generate a map, it is generaly used as a
 * singleton but has to be designed otherwise.
 *
 * Inside the Surface are Chunks that are 8x8 pixels, which can be either
 * pre-rendered, or generated on request - it is up to the individual Surface
 * to handle when that happens.
 */
export class Surface {
    private chunks = new WeakMap<WeakKey, Chunk>();
    private cache = new Set<WeakKey>();
    private oldCache: Set<WeakKey> | undefined;

    private timer: ReturnType<typeof setTimeout> | undefined;

    constructor(private land3D: NoiseFunction3D, private circumference: number, private zoom = 256) {
        // Setup
        this.cleanup = this.cleanup.bind(this);
    }

    /**
     * Should be called automatically on the first free cycle after we have
     * created a lot of chunks.
     */
    cleanup() {
        if (this.oldCache) {
            this.oldCache.difference(this.cache).forEach((key) => freeWeakKey(key));
        }
        this.oldCache = this.cache;
        this.cache = new Set();
        this.timer = undefined;
    }

    stats() {
        console.log("cache size", this.cache.size);
    }

    get(x: number, y: number) {
        x = Math.floor(x);
        y = Math.floor(y);
        const key = findWeakKey([x >>> 3, y >>> 3]);

        if (!this.cache.has(key)) {
            this.cache.add(key);
            if (!this.timer) {
                this.timer = setTimeout(this.cleanup);
            }
        }
        if (!this.chunks.has(key)) {
            getWeakKey(key); // Set the use count +1
            this.chunks.set(key, {
                x: x >>> 3,
                y: y >>> 3,
                height: new Array(8 * 8),
                jitter: new Array(8 * 8),
            });
        }
        const chunk = this.chunks.get(key)!;
        const offset = (x & 7) + (y & 7) * 8;

        let height = chunk.height[offset];
        let jitter = chunk.jitter[offset];

        if (height === undefined) {
            const land3D = this.land3D;

            const distance = this.circumference / this.zoom;

            const angle = (TAU * x) / this.circumference;
            const xPos = (Math.cos(angle) * distance) / TAU;
            const yPos = (Math.sin(angle) * distance) / TAU;
            const zPos = y / this.zoom;

            height = chunk.height[offset] = ((
                128 +
                land3D(xPos, yPos, zPos) * 64 +
                land3D(xPos * 2, yPos * 2, zPos * 2) * 32 +
                land3D(xPos * 4, yPos * 4, zPos * 4) * 16 +
                land3D(xPos * 8, yPos * 8, zPos * 8) * 8 +
                land3D(xPos * 16, yPos * 16, zPos * 16) * 4 +
                land3D(xPos * 32, yPos * 32, zPos * 32) * 2 +
                land3D(xPos * 64, yPos * 64, zPos * 64) * 1
            ) / 2);

            jitter = chunk.jitter[offset] = land3D(xPos * 32, yPos * 32, zPos * 32) * 8;
        }

        return { key, height, jitter, chunk };
    }
}
