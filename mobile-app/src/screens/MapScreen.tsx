import React, { useEffect, useState, useRef, useCallback } from 'react';
import { View, StyleSheet, TouchableOpacity, Text } from 'react-native';
import MapView, { Marker } from 'react-native-maps';
import * as Location from 'expo-location';
import { Ionicons } from '@expo/vector-icons';
import { useQuery } from '@tanstack/react-query';
import { fetchSightings } from '../api';
import type { Sighting } from '../types';
import LoadingSpinner from '../components/LoadingSpinner';
import OfflineMapBanner from '../components/OfflineMapBanner';
import TileDownloader from '../components/TileDownloader';
import { tileCache } from '../services/TileCache';
import { useNetworkStatus } from '../services/NetworkStatus';

const SPECIES_COLORS: Record<string, string> = {
  'Canis lupus': '#ef4444',
  'Canis latrans': '#f59e0b',
  'Canis familiaris': '#3b82f6',
};

function getColor(species: string): string {
  return SPECIES_COLORS[species] || '#8b5cf6';
}

interface MapRegion {
  latitude: number;
  longitude: number;
  latitudeDelta: number;
  longitudeDelta: number;
}

export default function MapScreen() {
  const [location, setLocation] = useState<{ latitude: number; longitude: number } | null>(null);
  const [showDownloader, setShowDownloader] = useState(false);
  const [mapRegion, setMapRegion] = useState<MapRegion | null>(null);
  const mapRef = useRef<MapView>(null);
  const { isConnected, isInternetReachable } = useNetworkStatus();
  const isOffline = isConnected === false || isInternetReachable === false;

  const { data: sightings } = useQuery({
    queryKey: ['sightings'],
    queryFn: fetchSightings,
  });

  useEffect(() => {
    (async () => {
      const { status } = await Location.requestForegroundPermissionsAsync();
      if (status === 'granted') {
        const loc = await Location.getCurrentPositionAsync({});
        const region = {
          latitude: loc.coords.latitude,
          longitude: loc.coords.longitude,
          latitudeDelta: 2,
          longitudeDelta: 2,
        };
        setLocation({ latitude: loc.coords.latitude, longitude: loc.coords.longitude });
        setMapRegion(region);
      } else {
        const region = { latitude: 44.4280, longitude: -110.5885, latitudeDelta: 2, longitudeDelta: 2 };
        setLocation({ latitude: 44.4280, longitude: -110.5885 });
        setMapRegion(region);
      }
    })();
  }, []);

  const handleRegionChange = useCallback((region: MapRegion) => {
    setMapRegion(region);
  }, []);

  const handleClearCache = useCallback(async () => {
    await tileCache.clearCache();
  }, []);

  if (!location) return <LoadingSpinner />;

  return (
    <View style={styles.container}>
      <OfflineMapBanner
        isOffline={isOffline}
        onPressDownload={() => setShowDownloader(true)}
        onPressClear={handleClearCache}
      />

      <MapView
        ref={mapRef}
        style={styles.map}
        initialRegion={{
          ...location,
          latitudeDelta: 2,
          longitudeDelta: 2,
        }}
        onRegionChangeComplete={handleRegionChange}
        showsUserLocation
      >
        {(sightings || []).map((s: Sighting, i: number) => (
          <Marker
            key={s.id || i}
            coordinate={{ latitude: s.latitude, longitude: s.longitude }}
            title={s.species}
            description={`${s.source} • ${new Date(s.observedOn).toLocaleDateString()}`}
            pinColor={getColor(s.species)}
          />
        ))}
      </MapView>

      <TouchableOpacity
        style={styles.downloadFab}
        onPress={() => setShowDownloader(true)}
      >
        <Ionicons name="download-outline" size={24} color="#fff" />
      </TouchableOpacity>

      <TileDownloader
        visible={showDownloader}
        onClose={() => setShowDownloader(false)}
        currentRegion={mapRegion ?? undefined}
        onDownloadComplete={() => {}}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  map: { flex: 1 },
  downloadFab: {
    position: 'absolute',
    bottom: 24,
    right: 16,
    backgroundColor: '#3b82f6',
    width: 52,
    height: 52,
    borderRadius: 26,
    alignItems: 'center',
    justifyContent: 'center',
    elevation: 5,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 3.84,
  },
});
