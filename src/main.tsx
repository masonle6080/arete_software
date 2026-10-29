import React from "react";
import ReactDOM from "react-dom/client";
import { printJobId } from "./export";
import { PrintScreen } from "./export/PrintScreen";
import { App } from "./shell/App";
import "./shell/app.css";

// A print window (PDF export) opens this page with `#print=ID`.
const printId = printJobId(window.location.hash);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>{printId ? <PrintScreen id={printId} /> : <App />}</React.StrictMode>,
);
