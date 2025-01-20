/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import classNames from "classnames";
import React, { FunctionComponent } from "react";

import { useSharedState } from "../../../hooks";
import { PlayerState } from "../../../Types";
import { Terrain } from "../../World/useTerrain";
import { useWorld } from "../../World/WorldContext";
import { useMapCoords } from "../useMapCoords";
import { Background } from "./Background";
import { Gate } from "./Feature/Gate";
import { Tree } from "./Feature/Tree";
import { useDungeon } from "./Floor/useDungeon";

import style from "./Cell.module.scss";

const terrainTypes: Record<Terrain, string> = {
    [Terrain.WATER]: style.terrainWater,
    [Terrain.SAND]: style.terrainSand,
    [Terrain.DIRT]: style.terrainGrass,
    [Terrain.ROCK]: style.terrainRock,
};

export interface ICell {
    x: number;
    y: number;
}

export const Cell: FunctionComponent<ICell> = ({ x, y }) => {
    const world = useWorld();
    const { circumference, gates } = world;
    const [playerX] = useSharedState<number>(PlayerState.X);
    const [playerY] = useSharedState<number>(PlayerState.Y);
    const [playerZ] = useSharedState<number>(PlayerState.Z);
    const [playerTravel] = useSharedState<number>(PlayerState.TRAVEL);
    const scale = playerTravel ? 100 : 1;
    const distanceX = x * scale;
    const distanceY = y * scale;
    const posX = distanceX + playerX;
    const posY = distanceY + playerY;
    const hasGate = playerZ >= 0 && gates.find(([x1, y1]) => posY === y1 && posX % circumference === x1);

    // console.log(`Cell ${playerX},${playerY},${playerZ}`, states.entries() );

    const mapGrid = useMapCoords(posX, posY, playerZ);
    const { background, feature, rain } = mapGrid;
    const terrainStyle = ""; //terrainTypes[background];
    const fog = distanceX * distanceX + distanceY * distanceY >= 1000 * 1000 ? 1 : 0;

    return (
        <div
            className={classNames(style.cell, terrainStyle, {
                [style.player]: !x && !y,
            })}
            style={{
                "--left": x,
                "--top": y,
                "--rain": fog ? 1 : rain * 0.1 + 1,
                "--fog": fog,
            }}
            onClick={() => {
                console.info(`Click: ${posX}, ${posY} (${x},${y})`, Terrain[background], mapGrid, world);
                useDungeon(playerX, playerY, playerZ);
            }}
        >
            <Background terrain={background} />
            {hasGate && <Gate />}
            {feature > 0.5 && <Tree />}
        </div>
    );
};
