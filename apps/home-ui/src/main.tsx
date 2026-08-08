import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "@limen/graphics/styles.css";
import "@limen/ui-kit/styles.css";
import "./styles.css";
import { App } from "./App";

const root = document.getElementById("root");

if (!root) {
  throw new Error("LIMEN Home root element was not found");
}

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
