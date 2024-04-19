/*
 * Copyright (c) 2024 Game. All rights reserved.
 */

import React from "react";
import { createRoot } from "react-dom/client";
import invariant from "tiny-invariant";

import { Game } from "./Game";

import "./index.css";

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
