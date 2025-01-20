/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { type Alea } from "../../../../common";
import { Direction, Tile } from "../../../../Types";
import type { Coordinate2D } from "../../../World/WorldContext";

const MIN_ROOM_SIZE = 3;
const MAX_ROOM_SIZE = 6;

const MIN_CORRIDOR_LENGTH = 3;
const MAX_CORRIDOR_LENGTH = 8;

const ROOM_CHANCE = 50; // corridorChance = 100 - roomChance

interface Rect {
    x: number;
    y: number;
    width: number;
    height: number;
}

export function DigRooms(rng: Alea, width: number, height: number) {
    const tiles: Tile[] = new Array(width * height).fill(Tile.UNUSED);
    const rooms: Rect[] = [];
    const exits: Rect[] = [];
    let up: Coordinate2D | undefined;
    let down: Coordinate2D | undefined;

    function generate() {
        const maxFeatures: number = Math.floor((width + height) / 2);

        // place the first room where we come in
        if (!makeRoom(Math.floor(width / 2), Math.floor(height / 2), Direction.DOWN, true)) {
            console.log("Unable to place the first room.");
            return;
        }

        // we already placed 1 feature (the first room)
        for (let i = 1; i < maxFeatures; ++i) {
            if (!createFeatures()) {
                console.log(`Unable to place more features (placed ${i}).`);
                break;
            }
        }

        up = placeObject(Tile.UPSTAIRS);
        down = placeObject(Tile.DOWNSTAIRS);

        deadend();
        trim();
    }

    function print() {
        let txt = "";

        console.log(`Dig - width: ${width}, height: ${height}`);
        for (let y = 0; y < height; ++y) {
            for (let x = 0; x < width; ++x) {
                txt += getTile(x, y);
            }
            txt += "\n";
        }
        console.log(txt);
    }

    function getTile(x: number, y: number) {
        if (x < 0 || y < 0 || x >= width || y >= height) {
            return Tile.UNUSED;
        }

        return tiles[x + y * width] ?? "?";
    }

    function setTile(x: number, y: number, tile: Tile) {
        tiles[x + y * width] = tile;
    }

    function deadend() {
        let changed = true;

        while (changed) {
            changed = false;
            for (let x = 1; x < width - 1; x++) {
                for (let y = 1; y < height - 1; y++) {
                    if (getTile(x, y) === Tile.CORRIDOOR) {
                        if (
                            (getTile(x - 1, y) === Tile.WALL ? 1 : 0) +
                                (getTile(x, y - 1) === Tile.WALL ? 1 : 0) +
                                (getTile(x + 1, y) === Tile.WALL ? 1 : 0) +
                                (getTile(x, y + 1) === Tile.WALL ? 1 : 0) ===
                            3
                        ) {
                            changed = true;
                            setTile(x, y, Tile.WALL);
                        }
                    }
                }
            }
            for (let x = 0; x < width; x++) {
                for (let y = 0; y < height; y++) {
                    if (
                        getTile(x, y) === Tile.WALL &&
                        (getTile(x - 1, y - 1) === Tile.WALL || getTile(x - 1, y - 1) === Tile.UNUSED) &&
                        (getTile(x, y - 1) === Tile.WALL || getTile(x, y - 1) === Tile.UNUSED) &&
                        (getTile(x + 1, y - 1) === Tile.WALL || getTile(x + 1, y - 1) === Tile.UNUSED) &&
                        (getTile(x - 1, y) === Tile.WALL || getTile(x - 1, y) === Tile.UNUSED) &&
                        (getTile(x + 1, y) === Tile.WALL || getTile(x + 1, y) === Tile.UNUSED) &&
                        (getTile(x - 1, y + 1) === Tile.WALL || getTile(x - 1, y + 1) === Tile.UNUSED) &&
                        (getTile(x, y + 1) === Tile.WALL || getTile(x, y + 1) === Tile.UNUSED) &&
                        (getTile(x + 1, y + 1) === Tile.WALL || getTile(x + 1, y + 1) === Tile.UNUSED)
                    ) {
                        changed = true;
                        setTile(x, y, Tile.UNUSED);
                    }
                }
            }
        }
    }

    function trim() {
        let minX = width;
        let minY = height;
        let maxX = 0;
        let maxY = 0;
        for (let x = 0; x < width; x++) {
            for (let y = 0; y < height; y++) {
                if (getTile(x, y) !== Tile.UNUSED) {
                    minX = Math.min(minX, x);
                    maxX = Math.max(maxX, x + 1);
                    minY = Math.min(minY, y);
                    maxY = Math.max(maxY, y + 1);
                }
            }
        }
        // console.log(`Size: ${width},${height} to ${maxX - minX},${maxY - minY} (${minX},${minY} -> ${maxX},${maxY})`)
        tiles.splice(maxY * width, -1);
        tiles.splice(0, minY * width);
        height = maxY - minY;
        for (let y1 = height - 1; y1 >= 0; y1--) {
            tiles.splice(y1 * width + maxX, width - maxX);
            tiles.splice(y1 * width, minX);
        }
        width = maxX - minX;
    }

    function createFeatures() {
        for (let i = 0; i < 1000; ++i) {
            if (!exits.length) {
                break;
            }

            let r = rng.uint(exits.length);
            let x = exits[r].width === 1 ? exits[r].x : rng.uint(exits[r].x + exits[r].width - 1, exits[r].x);
            let y = exits[r].height === 1 ? exits[r].y : rng.uint(exits[r].y + exits[r].height - 1, exits[r].y);

            if (
                createFeature(x, y, Direction.NORTH) ||
                createFeature(x, y, Direction.SOUTH) ||
                createFeature(x, y, Direction.EAST) ||
                createFeature(x, y, Direction.WEST)
            ) {
                exits.splice(r, 1);
                return true;
            }
        }

        return false;
    }

    function createFeature(x: number, y: number, dir: Direction) {
        const dx =
            dir === Direction.NORTH ? -1
            : dir === Direction.SOUTH ? 1
            : 0;
        const dy =
            dir === Direction.WEST ? -1
            : dir === Direction.EAST ? 1
            : 0;

        if (getTile(x + dx, y + dy) === Tile.FLOOR && getTile(x + dx, y + dy) === Tile.CORRIDOOR) {
            return false;
        }

        if (rng.uint(100) < ROOM_CHANCE) {
            if (makeRoom(x, y, dir)) {
                setTile(x, y, Tile.CLOSEDDOOR);

                return true;
            }
        } else if (makeCorridor(x, y, dir)) {
            if (getTile(x + dx, y + dy) === Tile.FLOOR) {
                setTile(x, y, Tile.CLOSEDDOOR);
            } else {
                // don't place a door between corridors
                setTile(x, y, Tile.CORRIDOOR);
            }

            return true;
        }

        return false;
    }

    function makeRoom(x: number, y: number, dir: Direction, firstRoom: boolean = false) {
        const width = rng.uint(MAX_ROOM_SIZE, MIN_ROOM_SIZE);
        const height = rng.uint(MAX_ROOM_SIZE, MIN_ROOM_SIZE);

        const room: Rect = {
            width,
            height,
            x: x - Math.floor(width / 2),
            y: y - Math.floor(height / 2),
        };

        if (dir === Direction.NORTH) {
            room.y = y - height;
        } else if (dir === Direction.SOUTH) {
            room.y = y + 1;
        } else if (dir === Direction.WEST) {
            room.x = x - width;
        } else if (dir === Direction.EAST) {
            room.x = x + 1;
        }

        if (placeRect(room, Tile.FLOOR)) {
            rooms.push(room);

            if (dir !== Direction.SOUTH || firstRoom) {
                // NORTH side
                exits.push({ x: room.x, y: room.y - 1, width: room.width, height: 1 });
            }
            if (dir !== Direction.NORTH || firstRoom) {
                // SOUTH side
                exits.push({ x: room.x, y: room.y + room.height, width: room.width, height: 1 });
            }
            if (dir !== Direction.EAST || firstRoom) {
                // WEST side
                exits.push({ x: room.x - 1, y: room.y, width: 1, height: room.height });
            }
            if (dir !== Direction.WEST || firstRoom) {
                // EAST side
                exits.push({ x: room.x + room.width, y: room.y, width: 1, height: room.height });
            }

            return true;
        }

        return false;
    }

    function makeCorridor(x: number, y: number, dir: Direction) {
        const corridor: Rect = {
            x,
            y,
            width: 0,
            height: 0,
        };

        if (rng.bool()) {
            // horizontal corridor
            corridor.width = rng.uint(MAX_CORRIDOR_LENGTH, MIN_CORRIDOR_LENGTH);
            corridor.height = 1;

            if (dir === Direction.NORTH) {
                corridor.y = y - 1;

                if (rng.bool()) {
                    // WEST
                    corridor.x = x - corridor.width + 1;
                }
            } else if (dir === Direction.SOUTH) {
                corridor.y = y + 1;

                if (rng.bool()) {
                    // WEST
                    corridor.x = x - corridor.width + 1;
                }
            } else if (dir === Direction.WEST) {
                corridor.x = x - corridor.width;
            } else if (dir === Direction.EAST) {
                corridor.x = x + 1;
            }
        } else {
            // vertical corridor
            corridor.width = 1;
            corridor.height = rng.uint(MAX_CORRIDOR_LENGTH, MIN_CORRIDOR_LENGTH);

            if (dir === Direction.NORTH) {
                corridor.y = y - corridor.height;
            } else if (dir === Direction.SOUTH) {
                corridor.y = y + 1;
            } else if (dir === Direction.WEST) {
                corridor.x = x - 1;

                if (rng.bool()) {
                    // NORTH
                    corridor.y = y - corridor.height + 1;
                }
            } else if (dir === Direction.EAST) {
                corridor.x = x + 1;

                if (rng.bool()) {
                    // NORTH
                    corridor.y = y - corridor.height + 1;
                }
            }
        }

        if (placeRect(corridor, Tile.CORRIDOOR)) {
            if (corridor.width !== 1) {
                if (dir !== Direction.SOUTH) {
                    exits.push({ x: corridor.x, y: corridor.y - 1, width: corridor.width, height: 1 });
                }
                if (dir !== Direction.NORTH) {
                    exits.push({ x: corridor.x, y: corridor.y + corridor.height, width: corridor.width, height: 1 });
                }
            }
            if (corridor.height !== 1) {
                if (dir !== Direction.EAST) {
                    exits.push({ x: corridor.x - 1, y: corridor.y, width: 1, height: corridor.height });
                }
                if (dir !== Direction.WEST) {
                    exits.push({ x: corridor.x + corridor.width, y: corridor.y, width: 1, height: corridor.height });
                }
            }

            return true;
        }

        return false;
    }

    function placeRect(rect: Rect, tile: Tile) {
        if (rect.x < 1 || rect.y < 1 || rect.x + rect.width > width - 1 || rect.y + rect.height > height - 1) {
            return false;
        }

        for (let y = rect.y; y < rect.y + rect.height; ++y) {
            for (let x = rect.x; x < rect.x + rect.width; ++x) {
                if (getTile(x, y) !== Tile.UNUSED) {
                    return false;
                }
            }
        }

        for (let y = rect.y - 1; y < rect.y + rect.height + 1; ++y) {
            for (let x = rect.x - 1; x < rect.x + rect.width + 1; ++x) {
                if (x === rect.x - 1 || y === rect.y - 1 || x === rect.x + rect.width || y === rect.y + rect.height) {
                    setTile(x, y, Tile.WALL);
                } else {
                    setTile(x, y, tile);
                }
            }
        }

        return true;
    }

    function placeObject(tile: Tile): Coordinate2D | undefined {
        if (rooms.length) {
            const r = rng.uint(rooms.length); // choose a random room
            const x = rng.uint(rooms[r].x + rooms[r].width - 2, rooms[r].x + 1);
            const y = rng.uint(rooms[r].y + rooms[r].height - 2, rooms[r].y + 1);

            if (getTile(x, y) === Tile.FLOOR) {
                setTile(x, y, tile);

                // place one object in one room (optional)
                rooms.splice(r, 1);

                return [y, y];
            }
        }
    }

    generate();

    if (!up) {
        console.log("Unable to place up stairs.");
    }
    if (!down) {
        console.log("Unable to place down stairs.");
    }

    print();

    return {
        tiles,
        width,
        height,
        up,
        down,
    };
}
