/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

export class PriorityArray<T> extends Array implements Array<T> {
    /**
     * Removes the last element from an array and returns it.
     * If the array is empty, undefined is returned and the array is not modified.
     */
    pop(): T | undefined {
        return super.pop()?.[1];
    };

    /**
     * Removes the first element from an array and returns it.
     * If the array is empty, undefined is returned and the array is not modified.
     */
    shift(): T | undefined {
        return super.shift()?.[1];
    };

    /**
     * Appends new elements to the end of an array, and returns the new length of the array.
     * @param item New element to add to the array.
     * @param priority Order for the new item.
     */
    put(item: T, priority: number) {
        let i = 0;

        while (super[i][0] < priority) {
            i++;
        }

        super.splice(i, 0, [priority, item]);

        return this;
    }
}
