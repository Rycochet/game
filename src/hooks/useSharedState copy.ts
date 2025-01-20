/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { useState, useEffect, useCallback, type Dispatch, type SetStateAction, useRef } from "react";

export type SharedKey = string | number | WeakKey;

export const states = new Map<SharedKey, any>();
const watchers = new Map<SharedKey, Set<(value: any) => void>>();
const references = new Map<SharedKey, number>();

const target = new EventTarget();
const EVENT = "set";

/**
 * Get the current value of the shared state. If you do not provide a default
 * value then it may return `undefined` if it has not been set before.
 *
 * @param key
 */
export function getSharedState<S = any>(key: string): S | undefined;
export function getSharedState<S = any>(key: string, def: S): S;
export function getSharedState<S = any>(key: string, def?: S) {
    return states.has(key) ? states.get(key) as S : def;
}

/**
 * Returns a shared stateful value, and a function to update it. If any of the
 * instances with the same key update then all will get the updated values.
 */
export function useSharedState<S>(key: SharedKey, initialState?: S | (() => S)): [S, Dispatch<SetStateAction<S>>] {
    const refKey = useRef(key);

    if (key !== refKey.current) {
        throw new TypeError("Cannot change key after instantiation.");
    }

    const hasInitialState = arguments.length > 1;
    const [state, setState] = useState<S>(states.has(key) ? states.get(key) : initialState);
    const setSharedState = useCallback((value: S) => {
        const oldValue = states.get(key);

        value = typeof value === "function"
            ? value(oldValue)
            : value;

        if (value !== oldValue) {
            states.set(key, value)
            setState(value);
            target.dispatchEvent(new CustomEvent(EVENT, { detail: { key, value } }))
        }
    }, []);

    useEffect(() => {
        references.set(key, (references.get(key) || 0) + 1);

        if (hasInitialState && !states.has(key)) {
            states.set(key, initialState);
            target.dispatchEvent(new CustomEvent(EVENT, { detail: { key, value: initialState } })), [];
        }

        const eventListener = (({ detail }: CustomEvent) => {
            if (detail.key === key) {
                setState(detail.value);
            }
        }) as EventListener;

        target.addEventListener(EVENT, eventListener, { passive: true });

        return () => {
            const count = references.get(key) || 1;

            if (count === 1) {
                states.delete(key);
                references.delete(key);
            } else {
                references.set(key, count - 1);
            }
            target.removeEventListener(EVENT, eventListener);
        }
    }, []);

    return [state, setSharedState as typeof setState];
}
