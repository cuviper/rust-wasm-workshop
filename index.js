import { Universe } from "./hello_world";

const pre = document.getElementById("game-of-life-canvas");
const universe = Universe.new();

const renderLoop = () => {
  universe.tick();
  pre.textContent = universe.render();
  requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);

// void async function renderLoop() {
//   setTimeout(() => requestAnimationFrame(renderLoop), 50);
//   universe.tick();
//   pre.textContent = universe.render();
// }();
