/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { FunctionComponent } from "react";

import { WorldMap } from "../WorldMap";
import { Map } from "./Map/Map";
import { Player } from "./Player/Player";
import { World } from "./World/World";

import "./Game.module.scss";

export const Game: FunctionComponent = () => {
    return (
        <World seed={12345}>
            <Player>
                <Map />
                <WorldMap />
            </Player>
        </World>
    );
};
