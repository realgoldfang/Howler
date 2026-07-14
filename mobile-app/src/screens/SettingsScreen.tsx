import React, { useState } from 'react';
import { View, Text, TextInput, ScrollView, TouchableOpacity, StyleSheet, Alert, Share } from 'react-native';
import * as FileSystem from 'expo-file-system';
import * as Sharing from 'expo-sharing';
import { Ionicons } from '@expo/vector-icons';
import { useMutation } from '@tanstack/react-query';
import { exportData } from '../api';
import { useApiKeys } from '../hooks/useApiKeys';

export default function SettingsScreen() {
  const { keys, updateKey, loaded } = useApiKeys();
  const [editing, setEditing] = useState<Record<string, string>>({});

  const handleChange = (key: string, value: string) => {
    setEditing((prev) => ({ ...prev, [key]: value }));
  };

  const handleSave = async (key: string) => {
    const value = editing[key] ?? '';
    await updateKey(key as any, value);
    setEditing((prev) => {
      const next = { ...prev };
      delete next[key];
      return next;
    });
    Alert.alert('Saved', `${key} updated`);
  };

  const exportMut = useMutation({
    mutationFn: async () => {
      const data = await exportData('json');
      const jsonStr = typeof data === 'string' ? data : JSON.stringify(data, null, 2);
      const fileUri = `${FileSystem.cacheDirectory}sightings.json`;
      await FileSystem.writeAsStringAsync(fileUri, jsonStr);
      if (await Sharing.isAvailableAsync()) {
        await Sharing.shareAsync(fileUri);
      } else {
        await Share.share({ message: jsonStr.slice(0, 1000) });
      }
    },
    onError: () => Alert.alert('Error', 'Export failed'),
  });

  if (!loaded) return null;

  const fields = [
    { key: 'serverUrl', label: 'Server URL', placeholder: 'http://192.168.1.x:8080', secure: false },
    { key: 'iucnToken', label: 'IUCN API Token', placeholder: 'Your IUCN Red List token', secure: true },
    { key: 'movebankUsername', label: 'Movebank Username', placeholder: 'Your Movebank username', secure: false },
    { key: 'movebankPassword', label: 'Movebank Password', placeholder: 'Your Movebank password', secure: true },
    { key: 'inaturalistToken', label: 'iNaturalist Token', placeholder: 'Your iNaturalist API token', secure: true },
  ] as const;

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.title}>Settings</Text>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>API Keys</Text>
        <Text style={styles.hint}>
          Configure your server URL and data source API keys. Keys are stored securely on your device.
        </Text>
        {fields.map(({ key, label, placeholder, secure }) => (
          <View key={key} style={styles.fieldGroup}>
            <Text style={styles.label}>{label}</Text>
            <View style={styles.inputRow}>
              <TextInput
                style={styles.input}
                placeholder={placeholder}
                placeholderTextColor="#475569"
                value={editing[key] !== undefined ? editing[key] : keys[key as keyof typeof keys]}
                onChangeText={(v) => handleChange(key, v)}
                secureTextEntry={secure}
                autoCapitalize="none"
                autoCorrect={false}
              />
              {editing[key] !== undefined && (
                <TouchableOpacity style={styles.saveBtn} onPress={() => handleSave(key)}>
                  <Ionicons name="checkmark-circle" size={28} color="#22c55e" />
                </TouchableOpacity>
              )}
            </View>
          </View>
        ))}
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>Export Data</Text>
        <TouchableOpacity
          style={styles.row}
          onPress={() => exportMut.mutate()}
          disabled={exportMut.isPending}
        >
          <Ionicons name="download-outline" size={20} color="#60a5fa" />
          <Text style={styles.rowText}>
            {exportMut.isPending ? 'Exporting...' : 'Export as JSON'}
          </Text>
        </TouchableOpacity>
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionTitle}>About</Text>
        <View style={styles.row}>
          <Ionicons name="information-circle-outline" size={20} color="#94a3b8" />
          <Text style={styles.rowText}>Howler Wolf Tracker v1.0.0</Text>
        </View>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0f172a' },
  content: { padding: 16 },
  title: { fontSize: 24, fontWeight: '700', color: '#f8fafc', marginBottom: 24 },
  section: { marginBottom: 24 },
  sectionTitle: { fontSize: 16, fontWeight: '600', color: '#94a3b8', marginBottom: 8, textTransform: 'uppercase', letterSpacing: 1 },
  hint: { fontSize: 13, color: '#64748b', marginBottom: 12, lineHeight: 18 },
  fieldGroup: { marginBottom: 12 },
  label: { fontSize: 14, color: '#cbd5e1', marginBottom: 4 },
  inputRow: { flexDirection: 'row', alignItems: 'center', gap: 8 },
  input: { flex: 1, backgroundColor: '#1e293b', color: '#f8fafc', borderRadius: 8, padding: 12, fontSize: 14 },
  saveBtn: { marginLeft: 4 },
  row: { flexDirection: 'row', alignItems: 'center', gap: 12, backgroundColor: '#1e293b', padding: 16, borderRadius: 8, marginBottom: 8 },
  rowText: { fontSize: 16, color: '#e2e8f0' },
});
