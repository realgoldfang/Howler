import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Modal,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { tileCache, type DownloadProgress } from '../services/TileCache';

interface TileDownloaderProps {
  visible: boolean;
  onClose: () => void;
  currentRegion?: {
    latitude: number;
    longitude: number;
    latitudeDelta: number;
    longitudeDelta: number;
  };
  onDownloadComplete?: () => void;
}

function estimateTiles(
  region: { latitudeDelta: number; longitudeDelta: number },
  minZoom: number,
  maxZoom: number
): number {
  let total = 0;
  for (let z = minZoom; z <= maxZoom; z++) {
    const scale = Math.pow(2, z);
    const tilesX = Math.ceil((region.longitudeDelta / 360) * scale);
    const tilesY = Math.ceil((region.latitudeDelta / 180) * scale);
    total += tilesX * tilesY;
  }
  return total;
}

function estimateSizeBytes(tileCount: number): number {
  return tileCount * 15_000; // ~15 KB per tile
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

export default function TileDownloader({
  visible,
  onClose,
  currentRegion,
  onDownloadComplete,
}: TileDownloaderProps) {
  const [minZoom, setMinZoom] = useState(8);
  const [maxZoom, setMaxZoom] = useState(12);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);

  const region = currentRegion ?? {
    latitude: 44.428,
    longitude: -110.5885,
    latitudeDelta: 2,
    longitudeDelta: 2,
  };

  const estimatedTiles = estimateTiles(region, minZoom, maxZoom);
  const estimatedSize = estimateSizeBytes(estimatedTiles);

  const handleDownload = useCallback(async () => {
    setDownloading(true);
    setProgress(null);

    const bounds = {
      north: region.latitude + region.latitudeDelta / 2,
      south: region.latitude - region.latitudeDelta / 2,
      east: region.longitude + region.longitudeDelta / 2,
      west: region.longitude - region.longitudeDelta / 2,
    };

    await tileCache.downloadRegion(
      bounds,
      { min: minZoom, max: maxZoom },
      'default',
      (p) => setProgress(p)
    );

    setDownloading(false);
    onDownloadComplete?.();
    onClose();
  }, [region, minZoom, maxZoom, onDownloadComplete, onClose]);

  function adjustMinZoom(delta: number) {
    setMinZoom((prev) => {
      const next = Math.max(1, Math.min(maxZoom, prev + delta));
      return next;
    });
  }

  function adjustMaxZoom(delta: number) {
    setMaxZoom((prev) => {
      const next = Math.max(minZoom, Math.min(19, prev + delta));
      return next;
    });
  }

  return (
    <Modal visible={visible} transparent animationType="slide">
      <View style={styles.overlay}>
        <View style={styles.modal}>
          <View style={styles.header}>
            <Text style={styles.title}>Download Map Tiles</Text>
            <TouchableOpacity onPress={onClose}>
              <Ionicons name="close" size={24} color="#94a3b8" />
            </TouchableOpacity>
          </View>

          <View style={styles.zoomRow}>
            <Text style={styles.label}>Min Zoom</Text>
            <View style={styles.stepper}>
              <TouchableOpacity onPress={() => adjustMinZoom(-1)} style={styles.stepBtn}>
                <Ionicons name="remove" size={20} color="#fff" />
              </TouchableOpacity>
              <Text style={styles.zoomValue}>{minZoom}</Text>
              <TouchableOpacity onPress={() => adjustMinZoom(1)} style={styles.stepBtn}>
                <Ionicons name="add" size={20} color="#fff" />
              </TouchableOpacity>
            </View>
          </View>

          <View style={styles.zoomRow}>
            <Text style={styles.label}>Max Zoom</Text>
            <View style={styles.stepper}>
              <TouchableOpacity onPress={() => adjustMaxZoom(-1)} style={styles.stepBtn}>
                <Ionicons name="remove" size={20} color="#fff" />
              </TouchableOpacity>
              <Text style={styles.zoomValue}>{maxZoom}</Text>
              <TouchableOpacity onPress={() => adjustMaxZoom(1)} style={styles.stepBtn}>
                <Ionicons name="add" size={20} color="#fff" />
              </TouchableOpacity>
            </View>
          </View>

          <View style={styles.estimate}>
            <Ionicons name="information-circle-outline" size={16} color="#94a3b8" />
            <Text style={styles.estimateText}>
              ~{estimatedTiles.toLocaleString()} tiles ({formatBytes(estimatedSize)})
            </Text>
          </View>

          {progress && (
            <View style={styles.progressContainer}>
              <View style={styles.progressBar}>
                <View
                  style={[
                    styles.progressFill,
                    {
                      width: `${progress.total > 0 ? (progress.downloaded / progress.total) * 100 : 0}%`,
                    },
                  ]}
                />
              </View>
              <Text style={styles.progressText}>
                {progress.downloaded}/{progress.total}
                {progress.failed > 0 ? ` (${progress.failed} failed)` : ''}
              </Text>
            </View>
          )}

          <TouchableOpacity
            style={[styles.downloadBtn, downloading && styles.downloadBtnDisabled]}
            onPress={handleDownload}
            disabled={downloading}
          >
            {downloading ? (
              <ActivityIndicator color="#fff" size="small" />
            ) : (
              <>
                <Ionicons name="download-outline" size={20} color="#fff" />
                <Text style={styles.downloadBtnText}>Download</Text>
              </>
            )}
          </TouchableOpacity>
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.6)',
    justifyContent: 'flex-end',
  },
  modal: {
    backgroundColor: '#1e293b',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    padding: 24,
    paddingBottom: 40,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 24,
  },
  title: {
    color: '#f8fafc',
    fontSize: 18,
    fontWeight: '600',
  },
  zoomRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  label: {
    color: '#cbd5e1',
    fontSize: 15,
  },
  stepper: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  stepBtn: {
    backgroundColor: '#334155',
    borderRadius: 6,
    width: 32,
    height: 32,
    alignItems: 'center',
    justifyContent: 'center',
  },
  zoomValue: {
    color: '#f8fafc',
    fontSize: 16,
    fontWeight: '600',
    width: 30,
    textAlign: 'center',
  },
  estimate: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginBottom: 20,
  },
  estimateText: {
    color: '#94a3b8',
    fontSize: 13,
  },
  progressContainer: {
    marginBottom: 20,
  },
  progressBar: {
    height: 6,
    backgroundColor: '#334155',
    borderRadius: 3,
    overflow: 'hidden',
    marginBottom: 6,
  },
  progressFill: {
    height: '100%',
    backgroundColor: '#3b82f6',
    borderRadius: 3,
  },
  progressText: {
    color: '#94a3b8',
    fontSize: 12,
    textAlign: 'center',
  },
  downloadBtn: {
    backgroundColor: '#3b82f6',
    borderRadius: 10,
    padding: 14,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
  },
  downloadBtnDisabled: {
    opacity: 0.6,
  },
  downloadBtnText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
