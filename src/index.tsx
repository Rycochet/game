/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React from "react";
import { createRoot } from "react-dom/client";
import invariant from "tiny-invariant";

import { Game } from "./Game";

// CSS Reset
import "./index.css";

declare module "react" {
    interface CSSProperties {
        [key: `--${string}`]: string | number;
    }
}

function init() {
    if (document.readyState === "complete") {
        const container = document.getElementById("root");
        invariant(container, "Unable to find the #root element!");

        createRoot(container).render(
            <React.StrictMode>
                <Game />
            </React.StrictMode>,
        );

        return true;
    }
}

if (!init()) {
    document.addEventListener("readystatechange", init);
}
