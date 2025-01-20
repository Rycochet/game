/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { FunctionComponent, useLayoutEffect, useMemo, useState } from "react";

import { Cell } from "./Cell/Cell";

import style from "./Map.module.scss";

export const Map: FunctionComponent = () => {
    // TODO: @react-hookz/web -> useMeasure
    const [size, setSize] = useState([0, 0]);
    useLayoutEffect(() => {
        const updateSize = () => setSize([document.body.clientWidth, document.body.clientHeight]);
        updateSize();
        window.addEventListener("resize", updateSize);
        return () => window.removeEventListener("resize", updateSize);
    }, []);

    // Count of cells, must be odd as `0` is the player in the center
    const width = useMemo(() => 2 * Math.ceil(size[0] / 64 / 2), [size]);
    const height = useMemo(() => 2 * Math.ceil(size[1] / 64 / 2), [size]);

    const cells = useMemo(() => {
        const arr: React.JSX.Element[] = [];
        const halfWidth = Math.floor(width / 2);
        const halfHeight = Math.floor(height / 2);

        for (let x = -halfWidth; x <= halfWidth; x++) {
            for (let y = -halfHeight; y <= halfHeight; y++) {
                arr.push(<Cell key={`${x},${y}`} x={x} y={y} />);
            }
        }

        return arr;
    }, [width, height]);

    return (
        <div
            className={style.map}
            style={{
                // Mass here to avoid rounding errors when using it in CSS
                "--deltaX": Math.floor(size[0] / 2),
                "--deltaY": Math.floor(size[1] / 2),
            }}
        >
            {cells}
        </div>
    );
};
