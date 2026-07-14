import * as FileSystem from 'expo-file-system';
import * as SQLite from 'expo-sqlite';

const TILE_BASE_URL = 'https://tile.openstreetmap.org';
const CACHE_DIR = `${FileSystem.documentDirectory}tile-cache`;
const DB_NAME = 'tile-cache.db';

interface CachedTile {
  tileUrl: string;
  localPath: string;
  downloadedAt: number;
  zoomLevel: number;
}

interface TileCoord {
  x: number;
  y: number;
  z: number;
}

function latLonToTile(lat: number, lon: number, zoom: number): TileCoord {
  const n = Math.pow(2, zoom);
  const x = Math.floor(((lon + 180) / 360) * n);
  const latRad = (lat * Math.PI) / 180;
  const y = Math.floor(((1 - Math.log(Math.tan(latRad) + 1 / Math.cos(latRad)) / Math.PI) / 2) * n);
  return { x, y, z: zoom };
}

function tilesInBounds(
  bounds: { north: number; south: number; east: number; west: number },
  zoom: number
): TileCoord[] {
  const topLeft = latLonToTile(bounds.north, bounds.west, zoom);
  const bottomRight = latLonToTile(bounds.south, bounds.east, zoom);
  const tiles: TileCoord[] = [];
  for (let x = topLeft.x; x <= bottomRight.x; x++) {
    for (let y = topLeft.y; y <= bottomRight.y; y++) {
      tiles.push({ x, y, z: zoom });
    }
  }
  return tiles;
}

function tileLocalPath(x: number, y: number, z: number, style: string): string {
  return `${CACHE_DIR}/${style}/${z}/${x}/${y}.png`;
}

function tileRemoteUrl(x: number, y: number, z: number): string {
  return `${TILE_BASE_URL}/${z}/${x}/${y}.png`;
}

let db: SQLite.SQLiteDatabase | null = null;

async function getDb(): Promise<SQLite.SQLiteDatabase> {
  if (!db) {
    db = await SQLite.openDatabaseAsync(DB_NAME);
    await db.execAsync(`
      CREATE TABLE IF NOT EXISTS tiles (
        tile_url TEXT PRIMARY KEY,
        local_path TEXT NOT NULL,
        downloaded_at INTEGER NOT NULL,
        zoom_level INTEGER NOT NULL
      );
    `);
  }
  return db;
}

export interface DownloadProgress {
  total: number;
  downloaded: number;
  failed: number;
  currentTile: string;
}

export type ProgressCallback = (progress: DownloadProgress) => void;

class TileCacheService {
  async downloadRegion(
    bounds: { north: number; south: number; east: number; west: number },
    zoomLevels: { min: number; max: number },
    style: string = 'default',
    onProgress?: ProgressCallback
  ): Promise<number> {
    const database = await getDb();
    const allTiles: TileCoord[] = [];

    for (let z = zoomLevels.min; z <= zoomLevels.max; z++) {
      allTiles.push(...tilesInBounds(bounds, z));
    }

    const progress: DownloadProgress = {
      total: allTiles.length,
      downloaded: 0,
      failed: 0,
      currentTile: '',
    };

    // Ensure cache directory exists
    await FileSystem.makeDirectoryAsync(CACHE_DIR, { intermediates: true });

    for (const tile of allTiles) {
      const url = tileRemoteUrl(tile.x, tile.y, tile.z);
      const localPath = tileLocalPath(tile.x, tile.y, tile.z, style);
      progress.currentTile = `${tile.z}/${tile.x}/${tile.y}`;

      // Skip if already cached
      const existing = await database.getFirstAsync<{ local_path: string }>(
        'SELECT local_path FROM tiles WHERE tile_url = ?',
        [url]
      );
      if (existing) {
        const info = await FileSystem.getInfoAsync(existing.local_path);
        if (info.exists) {
          progress.downloaded++;
          onProgress?.(progress);
          continue;
        }
      }

      // Create directory for tile
      const dirPath = localPath.substring(0, localPath.lastIndexOf('/'));
      await FileSystem.makeDirectoryAsync(dirPath, { intermediates: true });

      try {
        const downloadResult = await FileSystem.downloadAsync(url, localPath);
        if (downloadResult.status === 200) {
          await database.runAsync(
            'INSERT OR REPLACE INTO tiles (tile_url, local_path, downloaded_at, zoom_level) VALUES (?, ?, ?, ?)',
            [url, localPath, Date.now(), tile.z]
          );
          progress.downloaded++;
        } else {
          progress.failed++;
        }
      } catch {
        progress.failed++;
      }

      onProgress?.(progress);
    }

    return progress.downloaded;
  }

  async getTileUrl(x: number, y: number, z: number, style: string = 'default'): Promise<string> {
    const remoteUrl = tileRemoteUrl(x, y, z);
    const localPath = tileLocalPath(x, y, z, style);

    const info = await FileSystem.getInfoAsync(localPath);
    if (info.exists) {
      return localPath;
    }

    return remoteUrl;
  }

  async getCachedTileCount(): Promise<number> {
    const database = await getDb();
    const result = await database.getFirstAsync<{ count: number }>(
      'SELECT COUNT(*) as count FROM tiles'
    );
    return result?.count ?? 0;
  }

  async clearCache(): Promise<void> {
    const database = await getDb();
    await database.execAsync('DELETE FROM tiles');
    const info = await FileSystem.getInfoAsync(CACHE_DIR);
    if (info.exists) {
      await FileSystem.deleteAsync(CACHE_DIR, { idempotent: true });
    }
  }

  async getCacheSize(): Promise<number> {
    const database = await getDb();
    const rows = await database.getAllAsync<{ local_path: string }>(
      'SELECT local_path FROM tiles'
    );

    let totalSize = 0;
    for (const row of rows) {
      const info = await FileSystem.getInfoAsync(row.local_path);
      if (info.exists && info.size != null) {
        totalSize += info.size;
      }
    }
    return totalSize;
  }
}

export const tileCache = new TileCacheService();
export { latLonToTile, tilesInBounds, tileLocalPath, tileRemoteUrl };
