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

    const radius = this.getStationRadiusByZoom(map.getZoom());

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
        const r = radius * Math.sqrt(station.num);
        ctx.fillRect(p.x - r, p.y - r, r * 2, r * 2);
      }
    }
  }

  getStationRadiusByZoom(zoom: number) {
    switch (Math.floor(zoom)) {
      case 0:
        return 0.0015;
      case 1:
        return 0.005;
      case 2:
        return 0.01;
      case 3:
        return 0.02;
      case 4:
        return 0.05;
      case 5:
        return 0.1;
      case 6:
        return 0.15;
      case 7:
        return 0.25;
      case 8:
        return 0.35;
      case 9:
        return 0.5;
      case 10:
        return 0.8;
      case 11:
        return 1.2;
      case 12:
        return 1.5;
      default:
        return 2.5;
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
  const tilesNum = Math.pow(2, tileZoomLevel);

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
        tileX: mod(tileX, tilesNum),
        tileY: mod(tileY, tilesNum),
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

function mod(a: number, b: number) {
  return ((a % b) + b) % b;
}
