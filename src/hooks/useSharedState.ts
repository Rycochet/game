/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { useCallback, useEffect, useRef, useState, type Dispatch, type SetStateAction } from "react";

/**
 * Used as a key for a shared state. This allows direct string, as well as enum,
 * or even objects.
 */
export type SharedKey = string | number | WeakKey;

/**
 * This is the key:value map of shared data.
 */
const states = new Map<SharedKey, any>();

/**
 * This is a map of all setState functions for each key. When the watcher list
 * is empty the state can be deleted.
 */
const watchers = new Map<SharedKey, Set<(value: any) => void>>();

/**
 * Get the current value of the shared state. If you do not provide a default
 * value then it may return `undefined` if it has not been set before.
 */
export function getSharedState<S = any>(key: SharedKey): S | undefined;
export function getSharedState<S = any>(key: SharedKey, def: S): S;
export function getSharedState<S = any>(key: SharedKey, def?: S) {
    return states.has(key) ? (states.get(key) as S) : def;
}

/**
 * Set the current value of a state and ensure watchers are updated.
 */
export function setSharedState<S = any>(key: SharedKey, value: SetStateAction<S>) {
    const oldValue = states.get(key);

    if (value instanceof Function) {
        value = value(oldValue);
    }
    if (value !== oldValue) {
        states.set(key, value);
        watchers.get(key)?.forEach((set) => set(value));
    }
}

/**
 * Returns a shared stateful value, and a function to update it. If any of the
 * instances with the same key update then all will get the updated value.
 */
export function useSharedState<S>(key: SharedKey, initialState?: S | (() => S)): [S, Dispatch<SetStateAction<S>>] {
    const refKey = useRef(key);

    if (key !== refKey.current) {
        throw new Error("Cannot change key after instantiation.");
    }

    const hasInitialState = arguments.length > 1;
    const [state, setState] = useState<S>(states.has(key) ? states.get(key) : initialState);
    const setStateShared = useCallback((value: SetStateAction<S>) => setSharedState(key, value), []);

    useEffect(() => {
        if (hasInitialState && !states.has(key)) {
            // Set the shared initial state before we add our own watcher
            setSharedState(key, initialState instanceof Function ? initialState() : initialState);
        }
        // If we're the first watcher then make sure we can watch
        if (!watchers.has(key)) {
            watchers.set(key, new Set());
        }
        watchers.get(key)?.add(setState);

        return () => {
            watchers.get(key)?.delete(setState);

            // If no watchers left then delete the state
            if (!watchers.get(key)?.size) {
                states.delete(key);
                watchers.delete(key);
            }
        };
    }, []);

    return [state, setStateShared as typeof setState];
}
