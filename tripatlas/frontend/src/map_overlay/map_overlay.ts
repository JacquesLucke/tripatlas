import L from "leaflet";

let overlayContainer = document.getElementById(
  "map-overlay-container"
)! as HTMLDivElement;

export class MapOverlay {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
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
    const ctx = this.canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Canvas drawing not supported");
    }
    this.ctx = ctx;
    // gl.getExtension("OES_texture_float_linear");

    setTimeout(() => {
      this.render();
    }, 0);
  }

  render() {
    const ctx = this.ctx;
    const map = this.map;
    const mapSize = map.getSize();

    this.canvas.width = mapSize.x;
    this.canvas.height = mapSize.y;

    let canvasTiles = getCanvasTiles(map, 8, -100);

    ctx.clearRect(0, 0, mapSize.x, mapSize.y);
    ctx.strokeStyle = "#f00";
    ctx.lineWidth = 1;
    for (const canvasTile of canvasTiles) {
      ctx.beginPath();
      ctx.moveTo(canvasTile.left, canvasTile.top);
      ctx.lineTo(canvasTile.right, canvasTile.top);
      ctx.lineTo(canvasTile.right, canvasTile.bottom);
      ctx.lineTo(canvasTile.left, canvasTile.bottom);
      ctx.lineTo(canvasTile.left, canvasTile.top);
      ctx.stroke();
    }
  }
}

interface CanvasTileInfo {
  tileX: number;
  tileY: number;
  zoom: number;
  latLngBounds: L.LatLngBounds;

  left: number;
  right: number;
  top: number;
  bottom: number;
}

function getCanvasTiles(map: L.Map, tileZoomLevel: number, paddingPx: number) {
  const mapZoom = map.getZoom();
  const mapBounds = map.getBounds();
  const mapPixelBounds = map.getPixelBounds();
  const mapTopLeftPx = mapPixelBounds.getTopLeft();
  const crs = map.options.crs!;
  const tileSizePx = 256;

  const actualPaddingPx = paddingPx * Math.pow(2, tileZoomLevel - mapZoom);
  console.log(actualPaddingPx);

  const tileMin = crs
    .latLngToPoint(mapBounds.getNorthWest(), tileZoomLevel)
    .subtract([actualPaddingPx, actualPaddingPx])
    .divideBy(tileSizePx)
    .floor();
  const tileMax = crs
    .latLngToPoint(mapBounds.getSouthEast(), tileZoomLevel)
    .add([actualPaddingPx, actualPaddingPx])
    .divideBy(tileSizePx)
    .floor();

  let canvasTiles: CanvasTileInfo[] = [];
  for (let tileX = tileMin.x; tileX <= tileMax.x; tileX++) {
    for (let tileY = tileMin.y; tileY <= tileMax.y; tileY++) {
      const tileMinLatLng = crs.pointToLatLng(
        new L.Point(tileX, tileY).multiplyBy(tileSizePx),
        tileZoomLevel
      );
      const tileMaxLatLng = crs.pointToLatLng(
        new L.Point(tileX + 1, tileY + 1).multiplyBy(tileSizePx),
        tileZoomLevel
      );
      const tileMinCanvasPx = crs
        .latLngToPoint(tileMinLatLng, mapZoom)
        .subtract(mapTopLeftPx);
      const tileMaxCanvasPx = crs
        .latLngToPoint(tileMaxLatLng, mapZoom)
        .subtract(mapTopLeftPx);

      canvasTiles.push({
        tileX: tileX,
        tileY: tileY,
        zoom: tileZoomLevel,
        latLngBounds: new L.LatLngBounds([tileMinLatLng, tileMaxLatLng]),
        left: tileMinCanvasPx.x,
        right: tileMaxCanvasPx.x,
        top: tileMinCanvasPx.y,
        bottom: tileMaxCanvasPx.y,
      });
    }
  }

  return canvasTiles;
}
