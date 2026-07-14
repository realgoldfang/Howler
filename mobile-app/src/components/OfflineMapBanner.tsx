import React, { useEffect, useState } from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { tileCache } from '../services/TileCache';

interface OfflineMapBannerProps {
  isOffline: boolean;
  onPressDownload?: () => void;
  onPressClear?: () => void;
}

export default function OfflineMapBanner({
  isOffline,
  onPressDownload,
  onPressClear,
}: OfflineMapBannerProps) {
  const [tileCount, setTileCount] = useState(0);
  const [cacheSize, setCacheSize] = useState(0);

  useEffect(() => {
    loadStats();
  }, []);

  async function loadStats() {
    const count = await tileCache.getCachedTileCount();
    setTileCount(count);
    const size = await tileCache.getCacheSize();
    setCacheSize(size);
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
  }

  if (!isOffline) return null;

  return (
    <View style={styles.container}>
      <View style={styles.info}>
        <Ionicons name="cloud-offline-outline" size={20} color="#fff" />
        <Text style={styles.text}>
          Offline — {tileCount} tiles cached ({formatBytes(cacheSize)})
        </Text>
      </View>
      <View style={styles.actions}>
        <TouchableOpacity onPress={onPressDownload} style={styles.button}>
          <Ionicons name="download-outline" size={18} color="#fff" />
        </TouchableOpacity>
        {tileCount > 0 && (
          <TouchableOpacity onPress={onPressClear} style={styles.button}>
            <Ionicons name="trash-outline" size={18} color="#fca5a5" />
          </TouchableOpacity>
        )}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 50,
    left: 12,
    right: 12,
    backgroundColor: '#1e293b',
    borderRadius: 10,
    padding: 12,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    zIndex: 1000,
    elevation: 5,
  },
  info: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
    gap: 8,
  },
  text: {
    color: '#fff',
    fontSize: 13,
    flexShrink: 1,
  },
  actions: {
    flexDirection: 'row',
    gap: 8,
  },
  button: {
    padding: 6,
  },
});
