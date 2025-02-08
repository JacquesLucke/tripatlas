import L from "leaflet";

let overlayContainer = document.getElementById(
  "map-overlay-container"
)! as HTMLDivElement;

export class MapOverlay {
  canvas: HTMLCanvasElement;
  gl: WebGL2RenderingContext;
  map: L.Map;

  constructor(map: L.Map) {
    this.map = map;
    this.canvas = document.createElement("canvas");
    this.canvas.style.position = "absolute";
    this.canvas.style.top = "0";
    this.canvas.style.left = "0";
    this.canvas.style.height = "100vh";
    this.canvas.style.width = "100%";
    this.canvas.style.pointerEvents = "none";
    overlayContainer.appendChild(this.canvas);
    const gl = this.canvas.getContext("webgl2");
    if (!gl) {
      throw new Error("WebGL2 not supported");
    }
    this.gl = gl;
    gl.getExtension("OES_texture_float_linear");

    setTimeout(() => {
      this.render();
    }, 0);
  }

  render() {
    const map = this.map;
    const mapSize = map.getSize();

    this.canvas.width = mapSize.x;
    this.canvas.height = mapSize.y;

    const gl = this.gl;
    gl.viewport(0, 0, mapSize.x, mapSize.y);
    gl.clearColor(0, 0, 0, 0.5);
    gl.clear(gl.COLOR_BUFFER_BIT);
  }
}
