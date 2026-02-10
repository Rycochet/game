/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { useEffect, useLayoutEffect, useMemo, useRef, useState, type FunctionComponent } from "react";

import { hsl2rgb } from "../common";
import { useWorld } from "../Game/World/WorldContext";
import { useInterval, useSharedState } from "../hooks";
import { PlayerState } from "../Types";
import { Surface } from "./Surface";

export const COLOR_WATER_DEEP = "#000064";
export const COLOR_WATER = "hsl(232 100% 29%)"; // #001493

export const COLOR_GRASS_DARK = "#3c5400";
export const COLOR_GRASS = "hsl(77 60% 27%)"; // #586f1c
export const COLOR_GRASS_LIGHT = "#788c38";

export const COLOR_HILL_VERY_DARK = "#583800";
export const COLOR_HILL_DARK = "#74541c";
export const COLOR_HILL = "hsl(38, 44%, 39%)"; // #917038
export const COLOR_HILL_LIGHT = "#cca86f";

export const COLOR_MOUNTAIN_DARK = "#545454";
export const COLOR_MOUNTAIN = "hsl(0, 0%, 55%)"; // #8c8c8c
export const COLOR_MOUNTAIN_LIGHT = "#c4c4c4";

const TAU = 2 * Math.PI; // Radius ratio to world size

const MAX_MAP_WIDTH = 400;
const MAX_MAP_HEIGHT = 400;

const SCALE = 20;

let direction = -1;

export function setCoord(
    screenX: number,
    screenY: number,
    width: number,
    height: number,
    mapX: number,
    mapY: number,
    coordMap?: Uint32Array,
) {
    if (coordMap) {
        for (let left = 0; left < width; left++) {
            for (let top = 0; top < height; top++) {
                const offset = Math.floor(screenY + top) * (MAX_MAP_WIDTH * 2) + Math.floor(screenX + left) * 2;

                coordMap[offset] = mapX;
                coordMap[offset + 1] = mapY;
            }
        }
    }
}

export const WorldMap: FunctionComponent = () => {
    const { circumference, water, tilt, climate, land3D, weather3D } = useWorld();
    const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
    const [playerX, setPlayerX] = useSharedState<number>(PlayerState.X);
    const [playerY] = useSharedState<number>(PlayerState.Y);
    const [time] = useSharedState<number>("time");
    const tide = Math.cos((time * Math.PI) / 6) * (4 * tilt); // Earth axis 25 degrees gives 1m tides
    const coordMap = useRef<Uint32Array | undefined>(undefined); // screenX,screenY => mapX,mapY

    console.log("tide", time, tide);

    const surface = useMemo(() => new Surface(land3D, circumference / 1000), []);

    const [snow, setSnow] = useState(128);

    useEffect(() => {
        var ctx = canvas?.getContext("2d"); //, { alpha: false });

        if (ctx) {
            coordMap.current = new Uint32Array(ctx.canvas.width * ctx.canvas.height * 2);
        }
    }, [canvas]);

    // useInterval(() => setWeatherDelta((old) => old + 0.01), 100);

    // useInterval(() => {
    //     setSnow((old) => {
    //         if (old < 0) {
    //             direction = 1;
    //         } else if (old > 128) {
    //             direction = -1;
    //         }

    //         return old + direction;
    //     });
    //     setPlayerX((old) => old + 1);
    // }, 166);

    useEffect(() => {
        var ctx = canvas?.getContext("2d"); //, { alpha: false });

        if (!ctx) {
            return;
        }

        const imageData = ctx.createImageData(ctx.canvas.width, ctx.canvas.height);
        coordMap.current?.fill(0);

        console.time("Calculate surface");
        for (let x = 0; x < MAX_MAP_WIDTH; x++) {
            let top = MAX_MAP_HEIGHT + 64;
            let first = true;

            for (let y = MAX_MAP_HEIGHT; y >= 0; y -= 2) {
                const shiftX = (MAX_MAP_HEIGHT - y) / 6;

                if (x < shiftX || x > MAX_MAP_WIDTH - shiftX) {
                    continue;
                }

                const deltaX = (4 * shiftX + MAX_MAP_WIDTH) / MAX_MAP_WIDTH;
                const xPos = playerX + deltaX * (x - MAX_MAP_WIDTH / 2);
                const yPos = playerY + y - MAX_MAP_HEIGHT / 2;

                const { height, jitter } = surface.get(xPos, yPos);

                // TODO: Time of day
                const diff = (height - surface.get(xPos - 1, yPos).height) / 64;

                let h = 0;
                let s = 0;
                let l = 0;

                if (height + jitter > Math.min(snow, 90)) {
                    // #8c8c8c
                    h = 0;
                    s = 0;
                } else if (height + jitter > 80) {
                    // #917038
                    h = 36;
                    s = 0.4;
                } else if (height + Math.abs(jitter) > water + tide) {
                    if (height < water) {
                        // #e5e600
                        h = 60;
                        s = 0.7;
                    } else {
                        // #536e11
                        h = 80;
                        s = 0.4;
                    }
                } else {
                    // #001493
                    h = 232;
                    s = 1;
                }
                if (height > water + tide) {
                    l = (64 - h / 2 - diff * (2000 - s * 2000)) / 100;
                } else {
                    l = (height / 2 + Math.max(0, 0.5 - snow / 128) * 200 + 15) / 100;
                }

                let bottom = top;

                top = Math.ceil(y / 2 - Math.max(height, water + tide) / 2);

                if (first) {
                    first = false;

                    const waterBottom = Math.ceil(y / 2 - height / 2);

                    if (top !== waterBottom) {
                        hsl2rgb(h, s, l / 2, 1, imageData, x, top + 64, 1, waterBottom - top);
                    }
                    hsl2rgb(38, 1, 0.15, 1, imageData, x, waterBottom + 64, 1, Math.ceil(y / 2) - waterBottom);
                } else if (bottom > top) {
                    hsl2rgb(h, s, l, 1, imageData, x, top + 64, 1, bottom - top);
                    setCoord(x, top + 64, 1, bottom - top, playerX + y, playerY + y, coordMap.current);
                }
            }
        }
        console.timeEnd("Calculate surface");

        console.info(surface.get(playerX, playerY));
        surface.stats();

        ctx.putImageData(imageData, 0, 0);
    }, [canvas, playerX, playerY, snow, tide]);

    return (
        <canvas
            ref={setCanvas}
            width={MAX_MAP_WIDTH}
            height={Math.ceil(MAX_MAP_HEIGHT / 2) + 64}
            onClick={(e) => {
                // const offset = Math.floor(screenY + top) * (MAX_MAP_WIDTH * 2) + Math.floor(screenX + left) * 2;
                const offset = Math.floor(e.clientY / 2) * (MAX_MAP_WIDTH * 2) + Math.floor(e.clientX / 2) * 2;

                console.log(e.clientX, e.clientY, coordMap.current?.[offset], coordMap.current?.[offset + 1]);
            }}
            style={{
                position: "absolute",
                width: MAX_MAP_WIDTH * 2,
                height: MAX_MAP_HEIGHT + 64,
                zIndex: 1,
                background: "black",
                border: "0.5rem solid black",
                boxSizing: "content-box",
            }}
        />
    );
};
