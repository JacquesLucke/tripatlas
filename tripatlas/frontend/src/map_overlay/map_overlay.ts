import L from "leaflet";
import { getApiUrl } from "../api";
import pThrottle from "p-throttle";

let overlayContainer = document.getElementById(
  "map-overlay-container"
)! as HTMLDivElement;

const throttle = pThrottle({
  limit: 10,
  interval: 100,
});

interface StationGroup {
  lat: number;
  lon: number;
  num: number;
}

export class MapOverlay {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  map: L.Map;
  stationsByTile: Map<string, StationGroup[]> = new Map();
  renderScheduled: boolean = false;

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
    if (this.renderScheduled) {
      return;
    }
    this.renderScheduled = true;
    requestAnimationFrame(() => {
      this._render();
      this.renderScheduled = false;
    });
  }

  _render() {
    const ctx = this.ctx;
    const map = this.map;
    const mapSize = map.getSize();
    const crs = map.options.crs!;

    this.canvas.width = mapSize.x;
    this.canvas.height = mapSize.y;

    let canvasTiles = getCanvasTiles(map, Math.floor(map.getZoom()), 0);

    const radius = 0.5;

    ctx.clearRect(0, 0, mapSize.x, mapSize.y);
    ctx.lineWidth = 1;
    for (const canvasTile of canvasTiles) {
      const stations = this.getTileStations(
        canvasTile.tileX,
        canvasTile.tileY,
        canvasTile.zoom
      );
      for (const station of stations) {
        const pos = new L.LatLng(station.lat, station.lon);
        const p = crs
          .latLngToPoint(pos, map.getZoom())
          .subtract(map.getPixelBounds().getTopLeft());
        const r = radius * station.num;
        ctx.fillRect(p.x - r, p.y - r, r * 2, r * 2);
      }
    }
  }

  getTileStations(tileX: number, tileY: number, zoom: number) {
    const fallback: StationGroup[] = [];
    const tileKey = `${zoom}_${tileX}_${tileY}`;
    if (this.stationsByTile.has(tileKey)) {
      return this.stationsByTile.get(tileKey)!;
    }
    this.stationsByTile.set(tileKey, fallback);

    const fetchStations = throttle(async () => {
      const apiUrl = getApiUrl(
        `/some_hash_23434/${zoom}_${tileX}_${tileY}.json`
      );
      const response = await fetch(apiUrl);
      const stations = await response.json();
      this.stationsByTile.set(tileKey, stations);
      this.render();
    });

    fetchStations();
    return fallback;
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
