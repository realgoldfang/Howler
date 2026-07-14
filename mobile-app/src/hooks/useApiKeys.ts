import { useState, useEffect } from 'react';
import * as SecureStore from 'expo-secure-store';

const KEYS = {
  SERVER_URL: 'howler_server_url',
  IUCN_TOKEN: 'howler_iucn_token',
  MOVEBANK_USERNAME: 'howler_movebank_username',
  MOVEBANK_PASSWORD: 'howler_movebank_password',
  INATURALIST_TOKEN: 'howler_inaturalist_token',
} as const;

export interface ApiKeys {
  serverUrl: string;
  iucnToken: string;
  movebankUsername: string;
  movebankPassword: string;
  inaturalistToken: string;
}

const DEFAULTS: ApiKeys = {
  serverUrl: 'http://localhost:8080',
  iucnToken: '',
  movebankUsername: '',
  movebankPassword: '',
  inaturalistToken: '',
};

export function useApiKeys() {
  const [keys, setKeys] = useState<ApiKeys>(DEFAULTS);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    (async () => {
      const [serverUrl, iucnToken, movebankUsername, movebankPassword, inaturalistToken] =
        await Promise.all([
          SecureStore.getItemAsync(KEYS.SERVER_URL),
          SecureStore.getItemAsync(KEYS.IUCN_TOKEN),
          SecureStore.getItemAsync(KEYS.MOVEBANK_USERNAME),
          SecureStore.getItemAsync(KEYS.MOVEBANK_PASSWORD),
          SecureStore.getItemAsync(KEYS.INATURALIST_TOKEN),
        ]);

      setKeys({
        serverUrl: serverUrl || DEFAULTS.serverUrl,
        iucnToken: iucnToken || '',
        movebankUsername: movebankUsername || '',
        movebankPassword: movebankPassword || '',
        inaturalistToken: inaturalistToken || '',
      });
      setLoaded(true);
    })();
  }, []);

  const updateKey = async (key: keyof ApiKeys, value: string) => {
    const storeKey = KEYS[key.toUpperCase() as keyof typeof KEYS] || KEYS.SERVER_URL;
    await SecureStore.setItemAsync(storeKey, value);
    setKeys((prev) => ({ ...prev, [key]: value }));
  };

  const getHeaders = (): Record<string, string> => {
    const headers: Record<string, string> = {};
    if (keys.iucnToken) headers['X-IUCN-Token'] = keys.iucnToken;
    if (keys.movebankUsername) headers['X-Movebank-Username'] = keys.movebankUsername;
    if (keys.movebankPassword) headers['X-Movebank-Password'] = keys.movebankPassword;
    if (keys.inaturalistToken) headers['X-INaturalist-Token'] = keys.inaturalistToken;
    return headers;
  };

  return { keys, loaded, updateKey, getHeaders };
}
