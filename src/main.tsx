import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { windowControls } from "./ipc/window";
import "./styles.css";

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("#root bulunamadı");

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

// Tarayıcının varsayılan sağ tık menüsü (Yenile, İncele…) masaüstü uygulamasında gösterilmez;
// metin alanlarında kopyala/yapıştır için korunur.
window.addEventListener("contextmenu", (e) => {
  const t = e.target as HTMLElement | null;
  if (t?.closest("input, textarea, [contenteditable=true]")) return;
  e.preventDefault();
});

// Pencere görünmez başlar; ilk kare boyandıktan sonra gösterilir → beyaz flaş olmaz.
requestAnimationFrame(() => {
  requestAnimationFrame(() => {
    void windowControls.show();
  });
});
