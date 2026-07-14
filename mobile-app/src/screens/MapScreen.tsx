import React, { useEffect, useState } from 'react';
import { View, StyleSheet, Alert } from 'react-native';
import MapView, { Marker } from 'react-native-maps';
import * as Location from 'expo-location';
import { useQuery } from '@tanstack/react-query';
import { fetchSightings } from '../api';
import type { Sighting } from '../types';
import LoadingSpinner from '../components/LoadingSpinner';

const SPECIES_COLORS: Record<string, string> = {
  'Canis lupus': '#ef4444',
  'Canis latrans': '#f59e0b',
  'Canis familiaris': '#3b82f6',
};

function getColor(species: string): string {
  return SPECIES_COLORS[species] || '#8b5cf6';
}

export default function MapScreen() {
  const [location, setLocation] = useState<{ latitude: number; longitude: number } | null>(null);

  const { data: sightings } = useQuery({
    queryKey: ['sightings'],
    queryFn: () => fetchSightings({ limit: 200 }),
  });

  useEffect(() => {
    (async () => {
      const { status } = await Location.requestForegroundPermissionsAsync();
      if (status === 'granted') {
        const loc = await Location.getCurrentPositionAsync({});
        setLocation({ latitude: loc.coords.latitude, longitude: loc.coords.longitude });
      } else {
        setLocation({ latitude: 44.4280, longitude: -110.5885 });
      }
    })();
  }, []);

  if (!location) return <LoadingSpinner />;

  return (
    <View style={styles.container}>
      <MapView
        style={styles.map}
        initialRegion={{
          ...location,
          latitudeDelta: 2,
          longitudeDelta: 2,
        }}
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
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
  map: { flex: 1 },
});
