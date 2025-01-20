/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { FunctionComponent } from "react";

import image from "./tree1.png";

import style from "./Feature.module.scss";

export const Tree: FunctionComponent = () => <img className={style.feature} src={image} />;
