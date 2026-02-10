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
 * Finds a persistent WeakKey reference so we can use the same key from multiple
 * sources and not worry about them not being equal.
 *
 * This does not mark the key as being used so it will trigger the cleanup
 * check.
 */
export function findWeakKey<T extends WeakKey>(key: T) {
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
 * Finds a persistent WeakKey reference so we can use the same key from multiple
 * sources and not worry about them not being equal.
 *
 * This marks the key as being used, and must be matched by a `freeWeakKey`
 * call.
 */
export function getWeakKey<T extends WeakKey>(key: T) {
    const keyName = key.toString();

    if (keys.has(keyName)) {
        key = keys.get(keyName) as T;
        count.set(key, (count.get(key) || 0) + 1);
    } else {
        keys.set(keyName, key);
        count.set(key, 1);
    }

    return key;
}

/**
 * Reduces a WeakKey reference count, and triggers cleanup if nothing is using
 * it any more.
 */
export function freeWeakKey<T extends WeakKey>(key: T) {
    const keyName = key.toString();

    if (keys.has(keyName)) {
        key = keys.get(keyName) as T;

        // Ensure we don't go below 0
        const value = (count.get(key) || 1) - 1;

        count.set(key, value);
        if (!value) {
            triggerCleanup = true;
        }
    }
}

/**
 * Gives a persistent WeakKey reference so we can use the same key from multiple
 * sources and not worry about them not being equal.
 *
 * Wraps the key in a React hook to automatically free it when it goes out of
 * scope.
 */
export function useWeakKey<T extends WeakKey>(key: T) {
    // Ensure we free a key that goes out of scope.
    useEffect(() => {
        getWeakKey(key);

        return () => freeWeakKey(key)
    }, []);

    return findWeakKey(key);
};
