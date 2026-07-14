import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { Sighting } from '../types';

const SPECIES_ICONS: Record<string, keyof typeof Ionicons.glyphMap> = {
  'Canis lupus': 'paw',
  'Canis latrans': 'bug',
  'Canis familiaris': 'heart',
};

interface Props {
  sighting: Sighting;
  onPress?: () => void;
}

export default function SightingCard({ sighting, onPress }: Props) {
  const icon = SPECIES_ICONS[sighting.species] || 'eye';
  const date = new Date(sighting.observedOn).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });

  return (
    <TouchableOpacity style={styles.card} onPress={onPress} activeOpacity={onPress ? 0.7 : 1}>
      <View style={styles.iconWrap}>
        <Ionicons name={icon} size={24} color="#f59e0b" />
      </View>
      <View style={styles.body}>
        <View style={styles.header}>
          <Text style={styles.species} numberOfLines={1}>{sighting.species}</Text>
          {sighting.id && <Text style={styles.id}>#{sighting.id}</Text>}
        </View>
        {sighting.scientificName && (
          <Text style={styles.scientific}>{sighting.scientificName}</Text>
        )}
        <Text style={styles.detail} numberOfLines={1}>
          {sighting.source} • {date}
        </Text>
        <Text style={styles.coords}>
          {sighting.latitude.toFixed(4)}, {sighting.longitude.toFixed(4)}
        </Text>
        {sighting.details && (
          <Text style={styles.details} numberOfLines={2}>{sighting.details}</Text>
        )}
      </View>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  card: {
    flexDirection: 'row',
    backgroundColor: '#1e293b',
    borderRadius: 12,
    padding: 14,
    marginHorizontal: 16,
    marginVertical: 6,
  },
  iconWrap: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#334155',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  body: { flex: 1 },
  header: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center' },
  species: { fontSize: 16, fontWeight: '600', color: '#f8fafc', flex: 1 },
  id: { fontSize: 12, color: '#64748b', marginLeft: 8 },
  scientific: { fontSize: 13, color: '#94a3b8', fontStyle: 'italic', marginTop: 2 },
  detail: { fontSize: 13, color: '#94a3b8', marginTop: 4 },
  coords: { fontSize: 12, color: '#64748b', marginTop: 2, fontFamily: 'monospace' },
  details: { fontSize: 13, color: '#cbd5e1', marginTop: 6, lineHeight: 18 },
});
