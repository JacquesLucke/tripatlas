import L from "leaflet";
import "leaflet/dist/leaflet.css";

const mapContainer = document.getElementById("map-container")!;
const defaultCoordinates = new L.LatLng(52.637778, 13.203611);
const defaultZoom = 14;

const map = L.map(mapContainer, {
  // Allow arbitrary zoom levels.
  zoomSnap: 0,
  // Disable zoom animation for now, because it's hard to align with the
  // canvas overlay.
  zoomAnimation: false,
});

// Add the Leaflet specific attribution.
map.attributionControl.setPrefix(
  "<a href='https://leafletjs.com/'>Leaflet</a>"
);

// Initialize the current view of the canvas. Either from the default values
// or from the saved map view.
const savedMapView = JSON.parse(localStorage.getItem("map-view") ?? "{}");
map.setView(
  new L.LatLng(
    savedMapView.latitude ?? defaultCoordinates.lat,
    savedMapView.longitude ?? defaultCoordinates.lng
  ),
  savedMapView.zoom ?? defaultZoom
);

map.on("moveend", function () {
  // Save the current map view to local storage so that it can be restored.
  const center = map.getCenter();
  const zoom = map.getZoom();
  localStorage.setItem(
    "map-view",
    JSON.stringify({ latitude: center.lat, longitude: center.lng, zoom })
  );
});

// Add the OpenStreetMap background layer.
L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
  attribution:
    '<a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);
