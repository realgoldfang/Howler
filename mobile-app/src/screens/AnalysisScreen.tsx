import React, { useState } from 'react';
import { View, Text, ScrollView, TextInput, TouchableOpacity, StyleSheet, Alert } from 'react-native';
import { useMutation, useQuery } from '@tanstack/react-query';
import { analyzeSighting, fetchSightings } from '../api';
import type { Sighting } from '../types';
import LoadingSpinner from '../components/LoadingSpinner';

export default function AnalysisScreen() {
  const [idInput, setIdInput] = useState('');

  const { data: sightings } = useQuery({
    queryKey: ['sightings'],
    queryFn: fetchSightings,
  });

  const analysisMut = useMutation({
    mutationFn: (id: number) => analyzeSighting(id),
    onSuccess: (result) => Alert.alert('Analysis Complete', result),
    onError: () => Alert.alert('Error', 'Analysis failed'),
  });

  const runAnalysis = () => {
    const id = idInput.trim() ? parseInt(idInput.trim(), 10) : null;
    if (!id) {
      if (!idInput.trim() && sightings?.length) {
        setIdInput(String(sightings[0].id || ''));
        return Alert.alert('Pick a Sighting', 'Enter a sighting ID to analyze');
      }
      return Alert.alert('No ID', 'Enter a sighting ID');
    }
    analysisMut.mutate(id);
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.title}>Wolf Sighting Analysis</Text>

      <Text style={styles.label}>Sighting ID</Text>
      <TextInput
        style={styles.input}
        placeholder="e.g. 1"
        placeholderTextColor="#64748b"
        value={idInput}
        onChangeText={setIdInput}
        keyboardType="numeric"
      />

      <TouchableOpacity
        style={[styles.button, analysisMut.isPending && styles.buttonDisabled]}
        onPress={runAnalysis}
        disabled={analysisMut.isPending}
      >
        <Text style={styles.buttonText}>
          {analysisMut.isPending ? 'Analyzing...' : 'Run Analysis'}
        </Text>
      </TouchableOpacity>

      {analysisMut.isPending && <LoadingSpinner />}

      {analysisMut.data && (
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Result</Text>
          <Text style={styles.stat}>{analysisMut.data}</Text>
        </View>
      )}

      {analysisMut.isError && (
        <View style={styles.card}>
          <Text style={styles.cardTitle}>Error</Text>
          <Text style={styles.error}>{analysisMut.error?.message || 'Analysis failed'}</Text>
        </View>
      )}

      <View style={styles.card}>
        <Text style={styles.cardTitle}>Recent Sightings</Text>
        {(sightings || []).slice(0, 10).map((s: Sighting) => (
          <TouchableOpacity
            key={String(s.id)}
            style={styles.sightingRow}
            onPress={() => setIdInput(String(s.id || ''))}
          >
            <Text style={styles.sightingText}>
              #{s.id} — {s.species} ({s.source})
            </Text>
          </TouchableOpacity>
        ))}
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0f172a' },
  content: { padding: 16 },
  title: { fontSize: 24, fontWeight: '700', color: '#f8fafc', marginBottom: 16 },
  label: { fontSize: 14, color: '#94a3b8', marginBottom: 6 },
  input: { backgroundColor: '#1e293b', color: '#f8fafc', borderRadius: 8, padding: 12, fontSize: 16, marginBottom: 16 },
  button: { backgroundColor: '#2563eb', paddingVertical: 12, borderRadius: 8, alignItems: 'center', marginBottom: 16 },
  buttonDisabled: { opacity: 0.5 },
  buttonText: { color: '#fff', fontWeight: '600', fontSize: 16 },
  card: { backgroundColor: '#1e293b', borderRadius: 12, padding: 16, marginBottom: 16 },
  cardTitle: { fontSize: 18, fontWeight: '700', color: '#f8fafc', marginBottom: 12 },
  stat: { fontSize: 16, color: '#e2e8f0', marginBottom: 6 },
  error: { fontSize: 14, color: '#ef4444' },
  sightingRow: { paddingVertical: 8, borderBottomWidth: 1, borderBottomColor: '#334155' },
  sightingText: { fontSize: 14, color: '#cbd5e1' },
});
