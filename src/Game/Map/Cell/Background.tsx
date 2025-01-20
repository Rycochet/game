/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { FunctionComponent } from "react";

import { Terrain } from "../../World/useTerrain";
import dirt from "./Background/dirt.png";
import sand from "./Background/sand.png";
import rock from "./Background/stone.png";
import water from "./Background/water.png";

import "./Background.scss";

/*
blue_ice.png
dirt.png
grass.png
grass_block_top.png
gravel.png
ice.png
moss_block.png
packed_mud.png
podzol_top.png
sand.png
sandstone_bottom.png
sandstone_smooth.png
snow.png
stone.png
*/

const terrainTypes: Record<Terrain, string> = {
    [Terrain.WATER]: water,
    [Terrain.SAND]: sand,
    [Terrain.DIRT]: dirt,
    [Terrain.ROCK]: rock,
};

export const Background: FunctionComponent<{ terrain: Terrain }> = ({ terrain }) => (
    <img className={Terrain[terrain]} src={terrainTypes[terrain]} />
);
