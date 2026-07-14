import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Ionicons } from '@expo/vector-icons';

interface Props {
  message?: string;
  icon?: keyof typeof Ionicons.glyphMap;
}

export default function EmptyState({ message = 'No data', icon = 'folder-open-outline' }: Props) {
  return (
    <View style={styles.container}>
      <Ionicons name={icon} size={56} color="#334155" />
      <Text style={styles.text}>{message}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { alignItems: 'center', justifyContent: 'center', padding: 32 },
  text: { fontSize: 16, color: '#64748b', marginTop: 12 },
});
