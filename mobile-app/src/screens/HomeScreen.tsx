import React, { useState } from 'react';
import { View, FlatList, StyleSheet, RefreshControl } from 'react-native';
import { useQuery } from '@tanstack/react-query';
import { fetchSightings } from '../api';
import type { Sighting, FilterParams } from '../types';
import SightingCard from '../components/SightingCard';
import FilterBar from '../components/FilterBar';
import EmptyState from '../components/EmptyState';
import LoadingSpinner from '../components/LoadingSpinner';

export default function HomeScreen() {
  const [filters, setFilters] = useState<FilterParams>({});

  const { data, isLoading, refetch, isRefetching } = useQuery({
    queryKey: ['sightings', filters],
    queryFn: () => fetchSightings(filters),
  });

  if (isLoading) return <LoadingSpinner />;

  return (
    <View style={styles.container}>
      <FilterBar onChange={setFilters} />
      <FlatList
        data={data || []}
        keyExtractor={(item) => String(item.id || item.sourceId || Math.random())}
        renderItem={({ item }) => <SightingCard sighting={item} />}
        ListEmptyComponent={<EmptyState message="No sightings found" />}
        contentContainerStyle={data?.length === 0 ? styles.emptyContainer : undefined}
        refreshControl={<RefreshControl refreshing={isRefetching} onRefresh={refetch} tintColor="#2563eb" />}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#0f172a' },
  emptyContainer: { flex: 1, justifyContent: 'center' },
});
