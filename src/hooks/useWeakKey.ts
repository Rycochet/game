/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { useEffect } from "react";

const keys = new Map<string, WeakKey>();
const count = new Map<WeakKey, number>();

/**
 * Flag for if we should actually loop through and check if we can cleanup.
 */
let triggerCleanup = false;

// Defer deleting in case the user moves around, or we're building data for imminent use
setInterval(() => {
    if (triggerCleanup) {
        count.forEach((value, key) => {
            if (!value) {
                keys.delete(key.toString());
                count.delete(key);
            }
        });
        triggerCleanup = false;
    }
}, 5000);

/**
 * Gives a persistent WeakKey reference so we can use the same key from multiple
 * sources and not worry about them not being equal.
 */
export const getWeakKey = <T extends WeakKey>(key: T) => {
    const keyName = key.toString();

    if (keys.has(keyName)) {
        key = keys.get(keyName) as T;
    } else {
        keys.set(keyName, key);
        count.set(key, 0);
        triggerCleanup = true;
    }

    return key;
};

/**
 * Gives a persistent WeakKey reference so we can use the same key from multiple
 * sources and not worry about them not being equal.
 *
 * Wraps the key in a React hook to allow garbage collection.
 *
 * Cleanup is asynchronous allowing a short window in case we want to reuse old
 * data.
 */
export const useWeakKey = <T extends WeakKey>(key: T) => {
    key = getWeakKey(key);

    // Ensure we delete data that's out of scope again, side effect of letting
    // the cache WeakMap forget things
    useEffect(() => {
        count.set(key, (count.get(key) || 0) + 1);

        return () => {
            count.set(key, (count.get(key) || 1) - 1);
            triggerCleanup = true;
        };
    });

    return key;
};
