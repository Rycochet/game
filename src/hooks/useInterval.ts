/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import { useEffect, useRef } from "react"

/**
 * Repeatedly calls a function or executes a code snippet, with a fixed time
 * delay between each call. Automatically cleanup, and can change the delay
 * or function to `undefined` to prevent it being called.
 *
 * @param callback function to call
 * @param ms delay
 */
export function useInterval(callback?: () => void, ms?: number) {
    const savedCallback = useRef(callback);

    useEffect(() => {
        savedCallback.current = callback;
    }, [callback]);

    useEffect(() => {
        // The callback can change at any time, but don't setup if we don't have a delay.
        if (ms != null) {
            const id = setInterval(() => savedCallback.current?.(), ms);

            return () => clearInterval(id);
        }
    }, [ms]);
}
