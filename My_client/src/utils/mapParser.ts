export interface MapLayer {
  id: number;
  file: string;
  visible: boolean;
  minZoom: number;
  maxZoom: number;
  label?: string;
  brushColor?: number;
  penColor?: number;
}

export interface GeoSetInfo {
  name: string;
  projection: string;
  center: string;
  zoomLevel: number;
  layers: MapLayer[];
}

export function parseGeoSet(content: string): GeoSetInfo {
  const lines = content.split('\n');

  const info: GeoSetInfo = {
    name: '本地地图',
    projection: '',
    center: '0,0',
    zoomLevel: 10000,
    layers: [],
  };

  let layerIndex = 0;

  for (const line of lines) {
    const trimmed = line.trim();

    if (trimmed.startsWith('"\\GEOSET\\NAME"')) {
      const match = trimmed.match(/= "(.+?)"/);
      if (match) info.name = match[1];
    } else if (trimmed.startsWith('"\\GEOSET\\PROJECTION"')) {
      const match = trimmed.match(/= "(.*?)"/);
      if (match) info.projection = match[1];
    } else if (trimmed.startsWith('"\\GEOSET\\CENTER"')) {
      const match = trimmed.match(/= "(.+?)"/);
      if (match) info.center = match[1];
    } else if (trimmed.startsWith('"\\GEOSET\\ZOOMLEVEL"')) {
      const match = trimmed.match(/= "([\d.]+)/);
      if (match) info.zoomLevel = parseFloat(match[1]);
    } else if (trimmed.startsWith('"\\TABLE\\')) {
      const tableMatch = trimmed.match(/\\TABLE\\(\d+)/);
      if (tableMatch) {
        layerIndex = parseInt(tableMatch[1]);
      }

      const fileMatch = trimmed.match(/\\TABLE\\\d+\\FILE" = "(.+?)"/);
      if (fileMatch) {
        if (!info.layers[layerIndex - 1]) {
          info.layers[layerIndex - 1] = {
            id: layerIndex,
            file: fileMatch[1],
            visible: true,
            minZoom: 0,
            maxZoom: 100000,
          };
        }
      }

      const visibleMatch = trimmed.match(/\\TABLE\\\d+\\ISVISIBLE" = "(.+?)"/);
      if (visibleMatch && info.layers[layerIndex - 1]) {
        info.layers[layerIndex - 1].visible = visibleMatch[1] === 'TRUE';
      }

      const zoomMinMatch = trimmed.match(/\\TABLE\\\d+\\ZOOM\\MIN" = "([\d.]+)/);
      if (zoomMinMatch && info.layers[layerIndex - 1]) {
        info.layers[layerIndex - 1].minZoom = parseFloat(zoomMinMatch[1]);
      }

      const zoomMaxMatch = trimmed.match(/\\TABLE\\\d+\\ZOOM\\MAX" = "([\d.]+)/);
      if (zoomMaxMatch && info.layers[layerIndex - 1]) {
        info.layers[layerIndex - 1].maxZoom = parseFloat(zoomMaxMatch[1]);
      }
    }
  }

  return info;
}

export function getCenterCoordinates(center: string): { lng: number; lat: number } {
  const [lng, lat] = center.split(',').map(Number);
  return { lng, lat };
}

export function formatZoomLevel(zoom: number): string {
  if (zoom >= 1000000) return `${(zoom / 1000000).toFixed(1)}M`;
  if (zoom >= 1000) return `${(zoom / 1000).toFixed(1)}K`;
  return zoom.toFixed(0);
}


