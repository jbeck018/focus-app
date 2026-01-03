// mini-timer-main.tsx - Entry point for mini-timer window

import React from "react";
import ReactDOM from "react-dom/client";
import { MiniTimerWindow } from "./features/mini-timer/mini-timer-window";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MiniTimerWindow />
  </React.StrictMode>
);
