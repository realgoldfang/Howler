import React, { useState, useRef, useEffect } from 'react';
import { View, FlatList, StyleSheet, RefreshControl } from 'react-native';
import type { Sighting } from '../types';
import SightingCard from '../components/SightingCard';
import EmptyState from '../components/EmptyState';
import LoadingSpinner from '../components/LoadingSpinner';
import ConnectionBanner from '../components/ConnectionBanner';
import { useAutoFetch } from '../hooks/useAutoFetch';

export default function HomeScreen() {
  const [showBanner, setShowBanner] = useState(false);
  const prevOnlineRef = useRef<boolean | null>(null);

  const { data, isLoading, refetch, isOnline } = useAutoFetch(['sightings']);

  useEffect(() => {
    if (prevOnlineRef.current !== null && prevOnlineRef.current !== isOnline) {
      setShowBanner(true);
    }
    prevOnlineRef.current = isOnline;
  }, [isOnline]);

  if (isLoading) return <LoadingSpinner />;

  return (
    <View style={styles.container}>
      <ConnectionBanner isOnline={isOnline} showBanner={showBanner} />
      <FlatList
        data={data || []}
        keyExtractor={(item) => String(item.id || item.sourceId || Math.random())}
        renderItem={({ item }) => <SightingCard sighting={item} />}
        ListEmptyComponent={<EmptyState message="No sightings found" />}
        contentContainerStyle={data?.length === 0 ? styles.emptyContainer : undefined}
        refreshControl={<RefreshControl refreshing={false} onRefresh={refetch} tintColor="#2563eb" />}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0f172a' },
  emptyContainer: { flex: 1, justifyContent: 'center' },
});
