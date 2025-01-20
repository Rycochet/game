/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { useCallback, useEffect, useRef, useState, type FunctionComponent, type PropsWithChildren } from "react";

import { useSharedState } from "../../hooks";
import { Direction, PlayerState } from "../../Types";
import { useWorld } from "../World/WorldContext";

export const Player: FunctionComponent<PropsWithChildren> = ({ children }) => {
    const { gates, circumference } = useWorld();

    const lastDirection = useRef<Direction>(null);

    const [playerX, setPlayerX] = useSharedState(PlayerState.X, gates[0][0]);
    const [playerY, setPlayerY] = useSharedState(PlayerState.Y, gates[0][1]);
    const [playerZ, setPlayerZ] = useSharedState(PlayerState.Z, 0);
    const [playerTravel, setPlayerTravel] = useSharedState(PlayerState.TRAVEL, true);

    const playerMove = useCallback(
        (direction: Direction, distance: number = 1) => {
            const maxY = Math.floor(circumference / 5); // so we can't approach the poles

            switch (direction) {
                default:
                    return;
                case Direction.SOUTHWEST:
                    setPlayerX((old) => (old - distance + circumference) % circumference);
                    setPlayerY((old) => Math.min(maxY, old + distance));
                    break;
                case Direction.SOUTH:
                    setPlayerY((old) => Math.min(maxY, old + distance));
                    break;
                case Direction.SOUTHEAST:
                    setPlayerX((old) => (old + distance) % circumference);
                    setPlayerY((old) => Math.min(maxY, old + distance));
                    break;
                case Direction.WEST:
                    setPlayerX((old) => (old - distance + circumference) % circumference);
                    break;
                case Direction.EAST:
                    setPlayerX((old) => (old + distance) % circumference);
                    break;
                case Direction.NORTHWEST:
                    setPlayerX((old) => (old - distance + circumference) % circumference);
                    setPlayerY((old) => Math.max(-maxY, old - distance));
                    break;
                case Direction.NORTH:
                    setPlayerY((old) => Math.max(-maxY, old - distance));
                    break;
                case Direction.NORTHEAST:
                    setPlayerX((old) => (old + distance) % circumference);
                    setPlayerY((old) => Math.max(-maxY, old - distance));
                    break;

                case Direction.UP:
                    if (playerZ === 0 && playerTravel) {
                        setPlayerTravel(!playerTravel);
                    } else {
                        setPlayerZ((old) => Math.max(-1000, old - 1));
                    }
                    break;
                case Direction.DOWN:
                    if (playerZ === 0 && !playerTravel) {
                        setPlayerTravel(!playerTravel);
                    } else if (!playerTravel) {
                        setPlayerZ((old) => Math.max(-1000, old + 1));
                    }
                    break;
            }
            lastDirection.current = direction;
        },
        [circumference, playerTravel, playerZ],
    );

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (document.activeElement?.nodeName !== "INPUT") {
                const delta = (playerTravel ? 100 : 1) * (e.shiftKey ? 10 : 1);

                // console.log(e.code);
                switch (e.code) {
                    case "Numpad1":
                        playerMove(Direction.SOUTHWEST, delta);
                        break;
                    case "Numpad2":
                    case "ArrowDown":
                        playerMove(Direction.SOUTH, delta);
                        break;
                    case "Numpad3":
                        playerMove(Direction.SOUTHEAST, delta);
                        break;
                    case "Numpad4":
                    case "ArrowLeft":
                        playerMove(Direction.WEST, delta);
                        break;
                    case "Numpad6":
                    case "ArrowRight":
                        playerMove(Direction.EAST, delta);
                        break;
                    case "Numpad7":
                        playerMove(Direction.NORTHWEST, delta);
                        break;
                    case "Numpad8":
                    case "ArrowUp":
                        playerMove(Direction.NORTH, delta);
                        break;
                    case "Numpad9":
                        playerMove(Direction.NORTHEAST, delta);
                        break;

                    case "NumpadSubtract":
                        playerMove(Direction.DOWN, delta);
                        break;
                    case "NumpadAdd":
                        playerMove(Direction.UP, delta);
                        break;

                    default:
                        switch (e.key) {
                            case "w":
                            case "W":
                                playerMove(Direction.NORTH, delta);
                                break;
                            case "a":
                            case "A":
                                playerMove(Direction.WEST, delta);
                                break;
                            case "s":
                            case "S":
                                playerMove(Direction.SOUTH, delta);
                                break;
                            case "d":
                            case "D":
                                playerMove(Direction.EAST, delta);
                                break;

                            case ">":
                                playerMove(Direction.DOWN, delta);
                                break;
                            case "<":
                                playerMove(Direction.UP, delta);
                                break;

                            // Gates
                            case "1":
                                if (gates[0]) {
                                    setPlayerX(gates[0][0]);
                                    setPlayerY(gates[0][1]);
                                    break;
                                }
                                return;
                            case "2":
                                if (gates[1]) {
                                    setPlayerX(gates[1][0]);
                                    setPlayerY(gates[1][1]);
                                    break;
                                }
                                return;

                            default:
                                return;
                        }
                }
                e.preventDefault();
            }
        };
        document.addEventListener("keydown", handleKeyDown);

        return () => document.removeEventListener("keydown", handleKeyDown);
    });

    useEffect(() => {
        console.info(`Move: ${playerX}, ${playerY}, ${playerZ}, ${playerTravel}`);
    }, [playerX, playerY, playerZ, playerTravel]);

    return children;
};
