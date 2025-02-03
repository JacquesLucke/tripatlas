import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./index.css";

const apiUrl = import.meta.env.VITE_TRIP_ATLAS_API_URL ?? "./api";
console.log("Using API at:", apiUrl);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
