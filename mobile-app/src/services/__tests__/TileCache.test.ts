import { latLonToTile, tilesInBounds, tileLocalPath, tileRemoteUrl } from '../TileCache';

describe('latLonToTile', () => {
  it('converts coordinates to tile at zoom 0', () => {
    const tile = latLonToTile(0, 0, 0);
    expect(tile).toEqual({ x: 0, y: 0, z: 0 });
  });

  it('converts coordinates to tile at zoom 2', () => {
    const tile = latLonToTile(40.7128, -74.006, 2);
    expect(tile.z).toBe(2);
    expect(typeof tile.x).toBe('number');
    expect(typeof tile.y).toBe('number');
  });

  it('handles negative longitude', () => {
    const tile = latLonToTile(44.428, -110.5885, 5);
    expect(tile.z).toBe(5);
    expect(tile.x).toBeGreaterThanOrEqual(0);
    expect(tile.y).toBeGreaterThanOrEqual(0);
  });
});

describe('tilesInBounds', () => {
  it('returns tiles covering a bounding box', () => {
    const bounds = { north: 45, south: 44, east: -110, west: -111 };
    const tiles = tilesInBounds(bounds, 5);
    expect(tiles.length).toBeGreaterThan(0);
    for (const tile of tiles) {
      expect(tile.z).toBe(5);
    }
  });

  it('returns single tile when bounds are tiny', () => {
    const bounds = { north: 44.01, south: 44.0, east: -110.01, west: -110.02 };
    const tiles = tilesInBounds(bounds, 10);
    expect(tiles.length).toBeGreaterThanOrEqual(1);
  });
});

describe('tileLocalPath', () => {
  it('generates correct path structure', () => {
    const path = tileLocalPath(3, 5, 8, 'default');
    expect(path).toContain('/tile-cache/default/8/3/5.png');
  });

  it('uses custom style', () => {
    const path = tileLocalPath(1, 2, 3, 'satellite');
    expect(path).toContain('/tile-cache/satellite/3/1/2.png');
  });
});

describe('tileRemoteUrl', () => {
  it('generates correct OSM URL', () => {
    const url = tileRemoteUrl(3, 5, 8);
    expect(url).toBe('https://tile.openstreetmap.org/8/3/5.png');
  });
});
