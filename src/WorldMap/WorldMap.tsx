/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { useEffect, useLayoutEffect, useMemo, useState, type FunctionComponent } from "react";

import { useWorld } from "../Game/World/WorldContext";
import { useSharedState } from "../hooks";
import { PlayerState } from "../Types";

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

let direction = -0.01;

export const WorldMap: FunctionComponent = () => {
    const { circumference, climate, land3D, weather3D } = useWorld();
    const [canvas, setCanvas] = useState<HTMLCanvasElement | null>(null);
    const [playerX] = useSharedState<number>(PlayerState.X);
    const [playerY] = useSharedState<number>(PlayerState.Y);

    const [snow, setSnow] = useState(1.5);
    const [weatherDelta, setWeatherDelta] = useState(0);

    useLayoutEffect(() => {
        if (canvas) {
            canvas.width = Math.floor(canvas.offsetWidth / 2);
            canvas.height = Math.floor(canvas.offsetHeight / 2);
        }
    }, [canvas]);

    // useEffect(() => {
    //     const timer = setInterval(() => setWeatherDelta((old) => old + 0.01), 100);

    //     return () => clearInterval(timer);
    // }, []);

    // useEffect(() => {
    //     const timer = setInterval(
    //         () =>
    //             setSnow((old) => {
    //                 if (old < 0) {
    //                     direction = 0.01;
    //                 } else if (old > 1.5) {
    //                     direction = -0.01;
    //                 }
    //                 console.log(old + direction);

    //                 return old + direction;
    //             }),
    //         100,
    //     );

    //     return () => clearInterval(timer);
    // }, []);

    useEffect(() => {
        var ctx = canvas?.getContext("2d");

        if (!canvas || !ctx) {
            return;
        }

        console.time("Draw world map");

        // ctx.fillStyle = "#000000";
        // ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
        ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);

        const distance = circumference / 1000;

        for (let y = 0; y <= MAX_MAP_HEIGHT; y++) {
            let lastHeight = 0;

            const maxWidth = MAX_MAP_WIDTH + 0.6 * (y - MAX_MAP_HEIGHT / 2);
            const deltaY = (y * 100) / MAX_MAP_HEIGHT - 50;
            const left = (150 - maxWidth + MAX_MAP_WIDTH) / 2;
            const zPos = (playerY + deltaY * SCALE) / 1000;

            for (let x = 0; x < maxWidth; x++) {
                const deltaX = (x * 100) / maxWidth - 50;
                const angle = (TAU * (playerX + deltaX * SCALE)) / circumference;
                const xPos = (Math.cos(angle) * distance) / TAU;
                const yPos = (Math.sin(angle) * distance) / TAU;

                let height =
                    (land3D(xPos, yPos, zPos) +
                        land3D(xPos * 2, yPos * 2, zPos * 2) / 2 +
                        land3D(xPos * 4, yPos * 4, zPos * 4) / 4 +
                        land3D(xPos * 8, yPos * 8, zPos * 8) / 8 +
                        land3D(xPos * 16, yPos * 16, zPos * 16) / 16 +
                        land3D(xPos * 32, yPos * 32, zPos * 32) / 32 +
                        land3D(xPos * 64, yPos * 64, zPos * 64) / 64 +
                        land3D(xPos * 128, yPos * 128, zPos * 128) / 128 +
                        1) /
                    2;

                if (x > 0) {
                    const diff = height - lastHeight;
                    let jitter = land3D(xPos * 32, yPos * 32, zPos * 32) / 10;
                    let drawHeight = 4;

                    if (y + 2 > MAX_MAP_HEIGHT) {
                        drawHeight = Math.ceil(Math.max(0.3, height) * 20);
                        if (height < 0.3) {
                            ctx.fillStyle = `hsl(232 100% ${height * 50 + Math.max(0, 0.5 - snow) * 200 + 30}%)`; // #001493
                            ctx.fillRect(
                                left + x,
                                y / 2 + 30 - Math.ceil(0.3 * 20),
                                1,
                                Math.floor((0.3 - height) * 20),
                            );
                        }
                        ctx.fillStyle = "#000";
                        `hsl(38 100% 15%)`; // #583800
                        ctx.fillRect(left + x, y / 2 + 30 - Math.ceil(height * 20), 1, Math.ceil(height * 20) + 1);
                    } else {
                        // const weather = Math.max(0, weather3D(xPos + weatherDelta, yPos, zPos));

                        if (height + jitter > snow) {
                            // Snow from top down
                            ctx.fillStyle = `hsl(0 0% ${75 - diff * 2000}%)`; // #8c8c8c
                        } else if (height + jitter > 0.8) {
                            ctx.fillStyle = `hsl(0 0% ${55 - diff * 2000}%)`; // #8c8c8c
                        } else if (height + jitter > 0.6) {
                            ctx.fillStyle = `hsl(38 44% ${39 - diff * 1000}%)`; // #917038
                        } else if (height + Math.abs(jitter) > 0.3) {
                            ctx.fillStyle = `hsl(77 60% ${27 - diff * 1000}%)`; // #586f1c
                        } else if (height > 0.28) {
                            ctx.fillStyle = `hsl(60 100% ${45 - diff * 200}%)`; // #e5e600
                        } else {
                            ctx.fillStyle = `hsl(232 100% ${height * 50 + Math.max(0, 0.5 - snow) * 200 + 15}%)`; // #001493
                        }
                        height = Math.max(0.3, height);

                        ctx.fillRect(
                            left + x,
                            y / 2 + 30 - Math.ceil(height * 20),
                            1,
                            Math.min(Math.floor(height * 20), 4),
                        );

                        // if (weather >= 0) {
                        //     ctx.fillStyle = `hsl(0 0% 100% / ${weather / 2})`;
                        //     ctx.fillRect(left + x, y / 2, 1, 1);
                        // }
                    }
                }
                lastHeight = height;
            }
        }
        console.timeEnd("Draw world map");
    }, [canvas, playerX, playerY, snow, weatherDelta]);

    return (
        <canvas
            ref={setCanvas}
            width={MAX_MAP_WIDTH * 1.5}
            height={MAX_MAP_HEIGHT * 1.5}
            style={{
                position: "absolute",
                width: "100%",
                zIndex: 1,
            }}
        />
    );
};
