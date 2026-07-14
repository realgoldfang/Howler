import React, { useState } from 'react';
import { View, Text, ScrollView, TextInput, TouchableOpacity, StyleSheet, Alert } from 'react-native';
import { useMutation, useQuery } from '@tanstack/react-query';
import { analyzeMovement, analyzeTemporal, fetchTerritories, fetchSightings } from '../api';
import type { MovementAnalysis, TemporalAnalysis, Territory, Sighting } from '../types';
import LoadingSpinner from '../components/LoadingSpinner';

export default function AnalysisScreen() {
  const [idsInput, setIdsInput] = useState('');

  const { data: sightings } = useQuery({
    queryKey: ['sightings'],
    queryFn: () => fetchSightings({ limit: 500 }),
  });

  const parseIds = (): number[] => {
    if (idsInput.trim()) {
      return idsInput.split(',').map((s) => parseInt(s.trim(), 10)).filter(Boolean);
    }
    return (sightings || []).map((s: Sighting) => s.id!).filter(Boolean);
  };

  const movementMut = useMutation({
    mutationFn: analyzeMovement,
    onError: () => Alert.alert('Error', 'Movement analysis failed'),
  });

  const temporalMut = useMutation({
    mutationFn: analyzeTemporal,
    onError: () => Alert.alert('Error', 'Temporal analysis failed'),
  });

  const territoryMut = useMutation({
    mutationFn: fetchTerritories,
    onError: () => Alert.alert('Error', 'Territory clustering failed'),
  });

  const runAnalysis = (type: 'movement' | 'temporal' | 'territories') => {
    const ids = parseIds();
    if (!ids.length) return Alert.alert('No data', 'No sighting IDs available');
    if (type === 'movement') movementMut.mutate(ids);
    else if (type === 'temporal') temporalMut.mutate(ids);
    else territoryMut.mutate(ids);
  };

  const movement: MovementAnalysis | undefined = movementMut.data;
  const temporal: TemporalAnalysis | undefined = temporalMut.data;
  const territories: Territory[] | undefined = territoryMut.data;

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.title}>Wolf Movement Analysis</Text>

      <Text style={styles.label}>Sighting IDs (comma-separated, or leave empty for all)</Text>
      <TextInput
        style={styles.input}
        placeholder="e.g. 1,2,3"
        placeholderTextColor="#64748b"
        value={idsInput}
        onChangeText={setIdsInput}
      />

      <View style={styles.buttonRow}>
        <TouchableOpacity style={[styles.button, styles.blue]} onPress={() => runAnalysis('movement')} disabled={movementMut.isPending}>
          <Text style={styles.buttonText}>{movementMut.isPending ? 'Analyzing...' : 'Movement'}</Text>
        </TouchableOpacity>
        <TouchableOpacity style={[styles.button, styles.green]} onPress={() => runAnalysis('temporal')} disabled={temporalMut.isPending}>
          <Text style={styles.buttonText}>{temporalMut.isPending ? 'Analyzing...' : 'Temporal'}</Text>
        </TouchableOpacity>
        <TouchableOpacity style={[styles.button, styles.purple]} onPress={() => runAnalysis('territories')} disabled={territoryMut.isPending}>
          <Text style={styles.buttonText}>{territoryMut.isPending ? 'Clustering...' : 'Territories'}</Text>
        </TouchableOpacity>
      </View>

      {movementMut.isPending && <LoadingSpinner />}

      {movement && (
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Movement Analysis</Text>
          <Text style={styles.stat}>Total Distance: {movement.totalDistanceKm.toFixed(2)} km</Text>
          <Text style={styles.stat}>Average Speed: {movement.averageSpeedKmh.toFixed(2)} km/h</Text>
          <Text style={styles.stat}>Pattern: {movement.pattern}</Text>
          <Text style={styles.stat}>Movements: {movement.movements.length}</Text>
        </View>
      )}

      {temporal && (
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Temporal Analysis</Text>
          <Text style={styles.stat}>Most Active: {temporal.mostActivePeriod}</Text>
          {temporal.hourlyActivity.map((h) => (
            <View key={h.period} style={styles.barRow}>
              <Text style={styles.barLabel}>{h.period}</Text>
              <View style={[styles.bar, { width: `${h.percentage}%` }]} />
              <Text style={styles.barValue}>{h.count}</Text>
            </View>
          ))}
        </View>
      )}

      {territories && (
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Territory Clusters</Text>
          {territories.length === 0 ? (
            <Text style={styles.stat}>No territories found</Text>
          ) : (
            territories.map((t) => (
              <View key={t.id} style={styles.territory}>
                <Text style={styles.stat}>Territory #{t.id}</Text>
                <Text style={styles.detail}>Radius: {t.radiusKm.toFixed(2)} km • Sightings: {t.sightingCount}</Text>
                <Text style={styles.detail}>Species: {t.species.join(', ')}</Text>
              </View>
            ))
          )}
        </View>
      )}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0f172a' },
  content: { padding: 16 },
  title: { fontSize: 24, fontWeight: '700', color: '#f8fafc', marginBottom: 16 },
  label: { fontSize: 14, color: '#94a3b8', marginBottom: 6 },
  input: { backgroundColor: '#1e293b', color: '#f8fafc', borderRadius: 8, padding: 12, fontSize: 16, marginBottom: 16 },
  buttonRow: { flexDirection: 'row', gap: 8, marginBottom: 16 },
  button: { flex: 1, paddingVertical: 12, borderRadius: 8, alignItems: 'center' },
  blue: { backgroundColor: '#2563eb' },
  green: { backgroundColor: '#16a34a' },
  purple: { backgroundColor: '#7c3aed' },
  buttonText: { color: '#fff', fontWeight: '600', fontSize: 14 },
  card: { backgroundColor: '#1e293b', borderRadius: 12, padding: 16, marginBottom: 16 },
  cardTitle: { fontSize: 18, fontWeight: '700', color: '#f8fafc', marginBottom: 12 },
  stat: { fontSize: 16, color: '#e2e8f0', marginBottom: 6 },
  detail: { fontSize: 14, color: '#94a3b8', marginBottom: 4 },
  barRow: { flexDirection: 'row', alignItems: 'center', marginBottom: 6 },
  barLabel: { width: 60, fontSize: 12, color: '#94a3b8' },
  bar: { height: 12, backgroundColor: '#2563eb', borderRadius: 4, marginHorizontal: 8 },
  barValue: { fontSize: 12, color: '#e2e8f0', width: 30 },
  territory: { borderTopWidth: 1, borderTopColor: '#334155', paddingTop: 8, marginTop: 8 },
});
