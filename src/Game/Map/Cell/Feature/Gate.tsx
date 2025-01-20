/*
 * Copyright (c) 2025 Hall of Worlds. All rights reserved.
 */

import React, { FunctionComponent } from "react";

import image from "./abyssal_stair.png";

import style from "./Feature.module.scss";

export const Gate: FunctionComponent = () => <img className={style.feature} src={image} />;
