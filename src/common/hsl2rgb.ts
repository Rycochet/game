/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

/**
 * Translate color formats.
 *
 * From https://stackoverflow.com/a/64090995/3328079
 *
 * @param hue [0..360]
 * @param saturation [0..1]
 * @param lightness [0..1]
 * @param alpha [0..1]
 *
 * @param imageData Used for setting values without making a temporary array
 * @param x Offset inside the imageData
 * @param y Offset inside the imageData
 * @param width to fill a rect
 * @param height to fill a rect
 */
export function hsl2rgb(hue: number, saturation: number, lightness: number, alpha?: number): [red: number, green: number, blue: number];
export function hsl2rgb(hue: number, saturation: number, lightness: number, alpha: number, imageData: ImageData, x: number, y: number, width?: number, height?: number): void;
export function hsl2rgb(hue: number, saturation: number, lightness: number, alpha: number = 1, imageData?: ImageData, x?: number, y?: number, width: number = 1, height: number = 1) {
    const a = saturation * Math.min(lightness, 1 - lightness);
    const f = (n: number, k = (n + hue / 30) % 12) => lightness - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);

    if (imageData === undefined) {
        return [f(0), f(8), f(4), alpha];
    } else {
        const red = f(0) * 255;
        const green = f(8) * 255;
        const blue = f(4) * 255;

        alpha *= 255;

        for (let left = 0; left < width; left++) {
            for (let top = 0; top < height; top++) {
                const offset = Math.floor(y! + top) * (imageData.width * 4) + Math.floor(x! + left) * 4;

                imageData.data[offset] = red;
                imageData.data[offset + 1] = green;
                imageData.data[offset + 2] = blue;
                imageData.data[offset + 3] = alpha;
            }
        }
    }
}
