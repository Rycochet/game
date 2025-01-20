/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { createContext, useContext } from "react";
import type { NoiseFunction3D } from "simplex-noise";

/**
 * Coordinates for things.
 */
export type Coordinate2D = [number, number];
export type Coordinate3D = [number, number, number];

export interface IWorld {
    /**
     * The world seed, used as a basis for all generated content via the `alea`
     * random number generator.
     */
    seed: number;

    /**
     * The noise function for generating anything land based.
     */
    land3D: NoiseFunction3D;

    /**
     * Used for generating features. Each terrain type uses a Z index based on
     * type.
     */
    feature3D: NoiseFunction3D;

    /**
     * The noise function for generating anything weather based.
     */
    weather3D: NoiseFunction3D;

    /**
     * This is the radius of the planet (in meters), and used for generating the
     * landscape itself. Positions are clamped using this.
     *
     * @default 6378
     */
    radius: number;

    /**
     * Shortcut so we all use the same number (in meters).
     */
    circumference: number;

    /**
     * How long the year is, and hence how long the seasons are.
     *
     * @default 360
     */
    year: number;

    /**
     * How common ores are, this affects mining as well as weight on the surface
     * (ie, gravity) - some worlds might cause encumberance just by being there!
     *
     * @default 1
     */
    density: number;

    /**
     * Used to specify how "strong" the seasons are. With a low tilt there is
     * little to no change in weather, while a high tilt can result in half the
     * planet being snowed under. (Earth is 23.44 degrees)
     *
     * @default 0.25
     */
    tilt: number;

    /**
     * How rainy the planet is in general. Wet (higher) planets have more water,
     * dry planets have more mountains.
     *
     * @default 0 < n < 1
     */
    climate: number;

    /**
     * Gate locations, used for entry / exit to the world.
     */
    gates: [Coordinate2D, Coordinate2D];
}

export const WorldContext = createContext<IWorld>({} as any);

export const useWorld = () => useContext(WorldContext)
