import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Alert, Share } from 'react-native';
import * as FileSystem from 'expo-file-system';
import * as Sharing from 'expo-sharing';
import { Ionicons } from '@expo/vector-icons';
import { useMutation } from '@tanstack/react-query';
import { exportData, importData } from '../api';

export default function SettingsScreen() {
  const exportMut = useMutation({
    mutationFn: async (format: 'csv' | 'geojson' | 'kml') => {
      const data = await exportData(format);
      const fileUri = `${FileSystem.cacheDirectory}sightings.${format}`;
      await FileSystem.writeAsStringAsync(fileUri, data);
      if (await Sharing.isAvailableAsync()) {
        await Sharing.shareAsync(fileUri);
      } else {
        await Share.share({ message: data.slice(0, 1000) });
      }
    },
    onError: () => Alert.alert('Error', 'Export failed'),
  });

  const handleImport = async () => {
    try {
      const result = await importData({
        uri: 'file:///tmp/import.csv',
        name: 'import.csv',
        type: 'text/csv',
      });
      Alert.alert('Success', `Imported ${result.imported} records`);
    } catch {
      Alert.alert('Error', 'Import failed');
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Settings</Text>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Export Data</Text>
        {(['csv', 'geojson', 'kml'] as const).map((fmt) => (
          <TouchableOpacity key={fmt} style={styles.row} onPress={() => exportMut.mutate(fmt)}>
            <Ionicons name="download-outline" size={20} color="#60a5fa" />
            <Text style={styles.rowText}>Export as {fmt.toUpperCase()}</Text>
          </TouchableOpacity>
        ))}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Import Data</Text>
        <TouchableOpacity style={styles.row} onPress={handleImport}>
          <Ionicons name="cloud-upload-outline" size={20} color="#34d399" />
          <Text style={styles.rowText}>Import CSV/JSON</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>About</Text>
        <View style={styles.row}>
          <Text style={styles.rowText}>Howler Wolf Tracker v1.0.0</Text>
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0f172a', padding: 16 },
  title: { fontSize: 24, fontWeight: '700', color: '#f8fafc', marginBottom: 24 },
  section: { marginBottom: 24 },
  sectionTitle: { fontSize: 16, fontWeight: '600', color: '#94a3b8', marginBottom: 8, textTransform: 'uppercase', letterSpacing: 1 },
  row: { flexDirection: 'row', alignItems: 'center', gap: 12, backgroundColor: '#1e293b', padding: 16, borderRadius: 8, marginBottom: 8 },
  rowText: { fontSize: 16, color: '#e2e8f0' },
});
