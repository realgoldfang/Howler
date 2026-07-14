import React, { useEffect } from 'react';
import { StatusBar } from 'expo-status-bar';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { NavigationContainer } from '@react-navigation/native';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { Ionicons } from '@expo/vector-icons';
import HomeScreen from './screens/HomeScreen';
import MapScreen from './screens/MapScreen';
import AnalysisScreen from './screens/AnalysisScreen';
import SettingsScreen from './screens/SettingsScreen';
import type { RootTabParamList } from './types';
import { useApiKeys } from './hooks/useApiKeys';
import { setApiKeyHeaders, API_BASE } from './api';

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: 2, staleTime: 30000 } },
});

const Tab = createBottomTabNavigator<RootTabParamList>();

const ICON_MAP: Record<string, keyof typeof Ionicons.glyphMap> = {
  Home: 'paw',
  Map: 'map',
  Analysis: 'analytics',
  Settings: 'settings',
};

function AppNavigator() {
  const { keys, loaded } = useApiKeys();

  useEffect(() => {
    if (loaded) {
      setApiKeyHeaders({
        ...(keys.iucnToken && { 'X-IUCN-Token': keys.iucnToken }),
        ...(keys.movebankUsername && { 'X-Movebank-Username': keys.movebankUsername }),
        ...(keys.movebankPassword && { 'X-Movebank-Password': keys.movebankPassword }),
        ...(keys.inaturalistToken && { 'X-INaturalist-Token': keys.inaturalistToken }),
      });
    }
  }, [keys, loaded]);

  return (
    <NavigationContainer>
      <Tab.Navigator
        screenOptions={({ route }) => ({
          tabBarIcon: ({ color, size }) => (
            <Ionicons name={ICON_MAP[route.name] || 'ellipse'} size={size} color={color} />
          ),
          tabBarActiveTintColor: '#2563eb',
          tabBarInactiveTintColor: '#6b7280',
          headerStyle: { backgroundColor: '#1e293b' },
          headerTintColor: '#f8fafc',
        })}
      >
        <Tab.Screen name="Home" component={HomeScreen} options={{ title: 'Sightings' }} />
        <Tab.Screen name="Map" component={MapScreen} options={{ title: 'Map' }} />
        <Tab.Screen name="Analysis" component={AnalysisScreen} options={{ title: 'Analysis' }} />
        <Tab.Screen name="Settings" component={SettingsScreen} options={{ title: 'Settings' }} />
      </Tab.Navigator>
      <StatusBar style="light" />
    </NavigationContainer>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppNavigator />
    </QueryClientProvider>
  );
}
