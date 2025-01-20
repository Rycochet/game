/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

/// <reference types="node" />

// @ts-expect-error Missing declarations
import sri from "@small-tech/vite-plugin-sri";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import compression from "vite-plugin-compression2";
import simpleHtmlPlugin from "vite-plugin-simple-html";

export default defineConfig({
    build: {
        cssMinify: "lightningcss",
        outDir: "dist",
        assetsInlineLimit: 64000,
    },
    css: {
        transformer: "lightningcss",
        lightningcss: {
            cssModules: true,
        },
    },
    plugins: [
        react(),
        simpleHtmlPlugin({ minify: true }),
        compression(),
        // eslint-disable-next-line @typescript-eslint/no-unsafe-call
        sri(),
    ],
    server: {
        port: 3000,
        strictPort: true,
    },
});
