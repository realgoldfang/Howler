import React, { useState } from 'react';
import { View, Text, TextInput, TouchableOpacity, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { FilterParams } from '../types';

interface Props {
  onChange: (filters: FilterParams) => void;
}

const SOURCES = ['all', 'ebird', 'inaturalist', 'manual'];
const SPECIES_OPTIONS = ['all', 'Canis lupus', 'Canis latrans', 'Canis familiaris'];

export default function FilterBar({ onChange }: Props) {
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [source, setSource] = useState('all');
  const [species, setSpecies] = useState('all');
  const [expanded, setExpanded] = useState(false);

  const applyFilters = () => {
    const filters: FilterParams = {};
    if (startDate) filters.startDate = startDate;
    if (endDate) filters.endDate = endDate;
    if (source !== 'all') filters.source = source;
    if (species !== 'all') filters.species = species;
    onChange(filters);
  };

  const resetFilters = () => {
    setStartDate('');
    setEndDate('');
    setSource('all');
    setSpecies('all');
    onChange({});
  };

  return (
    <View style={styles.container}>
      <TouchableOpacity style={styles.toggle} onPress={() => setExpanded(!expanded)}>
        <Ionicons name="filter" size={16} color="#60a5fa" />
        <Text style={styles.toggleText}>Filters</Text>
        <Ionicons name={expanded ? 'chevron-up' : 'chevron-down'} size={16} color="#64748b" />
      </TouchableOpacity>

      {expanded && (
        <View style={styles.body}>
          <View style={styles.row}>
            <Text style={styles.label}>From</Text>
            <TextInput
              style={styles.input}
              placeholder="YYYY-MM-DD"
              placeholderTextColor="#475569"
              value={startDate}
              onChangeText={setStartDate}
            />
          </View>
          <View style={styles.row}>
            <Text style={styles.label}>To</Text>
            <TextInput
              style={styles.input}
              placeholder="YYYY-MM-DD"
              placeholderTextColor="#475569"
              value={endDate}
              onChangeText={setEndDate}
            />
          </View>

          <Text style={styles.label}>Source</Text>
          <View style={styles.chipRow}>
            {SOURCES.map((s) => (
              <TouchableOpacity
                key={s}
                style={[styles.chip, source === s && styles.chipActive]}
                onPress={() => setSource(s)}
              >
                <Text style={[styles.chipText, source === s && styles.chipTextActive]}>{s}</Text>
              </TouchableOpacity>
            ))}
          </View>

          <Text style={styles.label}>Species</Text>
          <View style={styles.chipRow}>
            {SPECIES_OPTIONS.map((sp) => (
              <TouchableOpacity
                key={sp}
                style={[styles.chip, species === sp && styles.chipActive]}
                onPress={() => setSpecies(sp)}
              >
                <Text style={[styles.chipText, species === sp && styles.chipTextActive]} numberOfLines={1}>
                  {sp === 'all' ? 'All' : sp.split(' ')[1]}
                </Text>
              </TouchableOpacity>
            ))}
          </View>

          <View style={styles.actions}>
            <TouchableOpacity style={styles.resetBtn} onPress={resetFilters}>
              <Text style={styles.resetText}>Reset</Text>
            </TouchableOpacity>
            <TouchableOpacity style={styles.applyBtn} onPress={applyFilters}>
              <Text style={styles.applyText}>Apply</Text>
            </TouchableOpacity>
          </View>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { backgroundColor: '#1e293b', paddingHorizontal: 16, paddingVertical: 10 },
  toggle: { flexDirection: 'row', alignItems: 'center', gap: 6 },
  toggleText: { fontSize: 14, color: '#60a5fa', fontWeight: '600' },
  body: { marginTop: 12 },
  row: { flexDirection: 'row', alignItems: 'center', marginBottom: 8 },
  label: { fontSize: 13, color: '#94a3b8', marginBottom: 4, marginTop: 4 },
  input: { flex: 1, backgroundColor: '#0f172a', color: '#f8fafc', borderRadius: 6, padding: 10, fontSize: 14, marginLeft: 8 },
  chipRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 6, marginBottom: 8 },
  chip: { paddingHorizontal: 12, paddingVertical: 6, borderRadius: 16, backgroundColor: '#0f172a' },
  chipActive: { backgroundColor: '#2563eb' },
  chipText: { fontSize: 13, color: '#94a3b8' },
  chipTextActive: { color: '#fff' },
  actions: { flexDirection: 'row', justifyContent: 'flex-end', gap: 8, marginTop: 8 },
  resetBtn: { paddingHorizontal: 16, paddingVertical: 8, borderRadius: 6, backgroundColor: '#334155' },
  resetText: { color: '#94a3b8', fontSize: 14, fontWeight: '600' },
  applyBtn: { paddingHorizontal: 16, paddingVertical: 8, borderRadius: 6, backgroundColor: '#2563eb' },
  applyText: { color: '#fff', fontSize: 14, fontWeight: '600' },
});
